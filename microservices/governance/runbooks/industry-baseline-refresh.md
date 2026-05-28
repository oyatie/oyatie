---
doc_class: Runbook
title: Industry Baseline Refresh
status: Accepted
date: 2026-05-20
microservice: governance
severity: sev2
audience: sre
owner_team: axis-foundry + council-architecture + ops-sre-reliability
source_wave: codex-runbooks-substrate-w1
change_scope: substance rewrite of thin existing runbook
doc_status: published
---

# Runbook: Industry Baseline Refresh

## Operator Contract
- Runbook id: governance-industry-baseline-refresh.
- Primary service namespace: `governance`.
- Owning rotation: PagerDuty oya-governance-primary; council-architecture reviewer-on-call.
- Incident channel: `#inc-governance-gates`.
- External dependencies: Cedar policy runtime maintainers; Wasmtime security list; GitHub Enterprise support.
- API authority: `https://governance.internal.oyatie.dev/v1/governance/industry-baseline-refresh/incident-handoff`.
- Audit event class: `EVT-GOVERNANCE-INDUSTRY_BASELINE_REFRESH-INCIDENT` with ADR-0263 fields `incident_id`, `tenant_id`, `cell_id`, `microservice`, `runbook_id`, `decision_id`, `evidence_hash`, `operator_id`.
- Stop condition: mitigation has held for 30 minutes, GovernanceIndustryBaselineRefreshCritical is green, and all handoff APIs in Cross-µservice Coordination return `202 accepted`.
- Safety invariant: never clear the incident until `EVT-GOVERNANCE-INDUSTRY_BASELINE_REFRESH-INCIDENT` is sealed and the postmortem skeleton exists under `evidence/postmortems/governance-industry-baseline-refresh-<incident-id>.md`.

## Trigger Conditions
- Page on alert `GovernanceIndustryBaselineRefreshCritical` when `oya_governance_industry_baseline_refresh_error_ratio > 0.02` for 10 minutes in any production cell.
- Page on alert `GovernanceIndustryBaselineRefreshSloBurn` when `oya_governance_industry_baseline_refresh_lag_seconds > 300` for 2 consecutive evaluator windows.
- Open a sev0 if `oya_governance_industry_baseline_refresh_correctness_ratio < 0.9999` and the affected label set includes `tenant_id` or `principal_id`.
- Open a sev1 if `oya_governance_industry_baseline_refresh_queue_depth > 5000` for 15 minutes or retry backlog grows by more than 20 percent in one 5 minute window.
- Trigger from customer report when Support tags the case `governance.industry-baseline-refresh.customer_visible` in Zendesk.
- Trigger from CI when `cargo run -p oya-dev-cli -- gate validate governance-industry-baseline-refresh --production-snapshot` exits non-zero against the latest production evidence bundle.
- Primary dashboard: `https://grafana.dev.oyatie.internal/d/governance-substrate/industry-baseline-refresh?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=109`.
- Secondary dashboard: `https://grafana.dev.oyatie.internal/d/governance-substrate/industry-baseline-refresh?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=210`.
- Loki explorer: `https://grafana.dev.oyatie.internal/explore?query={namespace="governance",runbook="industry-baseline-refresh"}`.
- Alertmanager route: `oyatie-governance-industry-baseline-refresh-critical`; silence only with incident commander approval and `EVT-GOVERNANCE-INDUSTRY_BASELINE_REFRESH-INCIDENT` evidence.
- Synthetic probe: `oya ops probe governance industry-baseline-refresh --cell prod-us-east-1 --tenant synthetic-canary` returns `healthy=true`.
- Drift detector: `registry/governance/industry-baseline-refresh/expected-state.json` hash differs from live `https://governance.internal.oyatie.dev/v1/admin/state-hash`.

## Symptoms
- User-facing impact: industry baseline refresh blocks or corrupts the governance control path for affected tenants.
- Operators see Grafana panel `lane-pass-rate / Industry Baseline Refresh burn rate` turn red before the primary alert resolves.
- Loki signature `governance.industry_baseline_refresh.incident_state=failed` appears with fields `incident_id`, `tenant_id`, `cell_id`, `decision_id`, `evidence_hash`.
- Kubernetes events include `reason=GovernanceIndustryBaselineRefreshDegraded` on deployment `governance-industry-baseline-refresh-worker`.
- Audit-chain shows missing or delayed `EVT-GOVERNANCE-INDUSTRY_BASELINE_REFRESH-INCIDENT` entries when queried with `oya audit-chain query --event-class EVT-GOVERNANCE-INDUSTRY_BASELINE_REFRESH-INCIDENT --since 30m`.
- Metric pattern: `oya_governance_industry_baseline_refresh_error_ratio` rises before `oya_governance_industry_baseline_refresh_lag_seconds`; if lag rises first, suspect dependency saturation rather than local regression.
- Metric pattern: `oya_governance_industry_baseline_refresh_queue_depth` increases while pod CPU stays below 40 percent; suspect downstream refusal or feature flag deadlock.
- Tenant-specific shape: one `tenant_id` dominates labels in `oya_governance_industry_baseline_refresh_queue_depth`; isolate before fleet mitigation.
- Fleet-wide shape: at least three cells report `GovernanceIndustryBaselineRefreshCritical` in one 15 minute window; switch to sev1 bridge even if individual tenants are low-volume.
- Log signature `decision=deny reason=industry-baseline-refresh.policy_guard` means the guard is working; investigate caller inputs before rollback.
- Log signature `decision=permit reason=industry-baseline-refresh.break_glass` means manual intervention is active; confirm two-person authorization.
- Log signature `audit_emit_status=stalled event_class=EVT-GOVERNANCE-INDUSTRY_BASELINE_REFRESH-INCIDENT` means mitigation cannot be closed until replay succeeds.

## Diagnostic Steps
1. Set incident variables: `export INCIDENT_ID=INC-governance-industry-baseline-refresh-$(date -u +%Y%m%dT%H%M%SZ); export CELL=prod-us-east-1; export TENANT=synthetic-canary`.
2. Confirm active alerts: `curl -s https://governance.internal.oyatie.dev/v1/alerts?runbook=industry-baseline-refresh | jq .alerts`.
3. Check Kubernetes rollout: `kubectl -n governance rollout status deploy/governance-industry-baseline-refresh-worker --timeout=60s`.
4. List unhealthy pods: `kubectl -n governance get pods -l app=governance-industry-baseline-refresh -o wide`.
5. Read structured logs: `kubectl -n governance logs deploy/governance-industry-baseline-refresh-worker --since=30m | rg "governance.industry_baseline_refresh.incident_state|GovernanceIndustryBaselineRefreshCritical|EVT-GOVERNANCE-INDUSTRY_BASELINE_REFRESH-INCIDENT"`.
6. Query Loki directly: `logcli query '{namespace="governance",runbook="industry-baseline-refresh"}' --since=30m --limit=200`.
7. Check Prometheus fast burn: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_governance_industry_baseline_refresh_error_ratio{cell="prod-us-east-1"}'`.
8. Check lag: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_governance_industry_baseline_refresh_lag_seconds{cell="prod-us-east-1"}'`.
9. Check queue: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_governance_industry_baseline_refresh_queue_depth{cell="prod-us-east-1"}'`.
10. Open primary dashboard: `open "https://grafana.dev.oyatie.internal/d/governance-substrate/industry-baseline-refresh?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=109&var-incident=$INCIDENT_ID"`.
11. Open secondary dashboard: `open "https://grafana.dev.oyatie.internal/d/governance-substrate/industry-baseline-refresh?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=210&var-tenant=$TENANT"`.
12. Verify audit-chain emission: `oya audit-chain query --event-class EVT-GOVERNANCE-INDUSTRY_BASELINE_REFRESH-INCIDENT --since 30m --cell $CELL --tenant $TENANT`.
13. Verify service state: `oya ops governance industry-baseline-refresh status --cell $CELL --tenant $TENANT --output json`.
14. Run production snapshot gate: `cargo run -p oya-dev-cli -- gate validate governance-industry-baseline-refresh --production-snapshot --cell $CELL`.
15. Check Cargo owner crate: `cargo test -p oya-governance-domain industry_baseline_refresh -- --nocapture`.
16. Check API contract smoke: `curl -s https://governance.internal.oyatie.dev/v1/governance/industry-baseline-refresh/incident-handoff -H "x-oya-tenant: $TENANT"`.
17. Inspect config: `kubectl -n governance get configmap governance-industry-baseline-refresh-config -o yaml`.
18. Inspect feature flags: `oya flags get oya.governance.industry_baseline_refresh.incident_hold --cell $CELL --tenant $TENANT --output yaml`.
19. Inspect circuit breaker: `oya ops breaker status governance-industry-baseline-refresh-circuit-breaker --cell $CELL --tenant $TENANT`.
20. Check recent deploy: `kubectl -n governance rollout history deploy/governance-industry-baseline-refresh-worker | tail -20`.
21. Check policy file: `test -f microservices/governance/policy/lane-execution.cedar || test -f microservices/governance/policy/lane-execution.md`.
22. Check SLO files: `ls microservices/governance/slos/*.openslo.yaml | sort`.
23. Check catalog components: `find microservices/governance/catalog -maxdepth 1 -type f | sort | rg "governance|industry"`.
24. Confirm no cross-cell spread: `oya ops cells query --metric oya_governance_industry_baseline_refresh_error_ratio --window 30m --threshold 0.02`.
25. Snapshot evidence: `oya evidence snapshot --incident $INCIDENT_ID --microservice governance --runbook industry-baseline-refresh --output evidence/incidents/$INCIDENT_ID.json`.

### Diagnostic Decision Tree
```text
Industry Baseline Refresh incident decision tree
1. Is GovernanceIndustryBaselineRefreshCritical firing in more than one cell?
   |-- yes: declare fleet incident, page PagerDuty oya-governance-primary; council-architecture reviewer-on-call, and run cross-cell containment.
   |-- no: keep scope to the affected cell and continue tenant isolation checks.
2. Does oya_governance_industry_baseline_refresh_queue_depth grow while oya_governance_industry_baseline_refresh_error_ratio is flat?
   |-- yes: downstream dependency or replay backlog; choose mitigation branch B.
   |-- no: local regression or bad input; continue branch selection.
3. Does audit-chain show EVT-GOVERNANCE-INDUSTRY_BASELINE_REFRESH-INCIDENT gaps?
   |-- yes: do not close; run evidence replay before resolution.
   |-- no: mitigation can proceed after state is green.
4. Is customer or regulator impact confirmed?
   |-- yes: promote severity, open #inc-governance-gates, and notify compliance handoff.
   |-- no: keep internal incident and collect evidence.
```
- Branch A (backlog with healthy dependencies): use the matching mitigation block below and record `decision_branch=A` in `EVT-GOVERNANCE-INDUSTRY_BASELINE_REFRESH-INCIDENT`.
- Branch B (downstream dependency degraded): use the matching mitigation block below and record `decision_branch=B` in `EVT-GOVERNANCE-INDUSTRY_BASELINE_REFRESH-INCIDENT`.
- Branch C (bad deploy introduced regression): use the matching mitigation block below and record `decision_branch=C` in `EVT-GOVERNANCE-INDUSTRY_BASELINE_REFRESH-INCIDENT`.
- Branch D (data replay or rebuild required): use the matching mitigation block below and record `decision_branch=D` in `EVT-GOVERNANCE-INDUSTRY_BASELINE_REFRESH-INCIDENT`.

## Mitigation Steps
1. Acknowledge page: `pd incident ack --service governance --incident $INCIDENT_ID`.
2. Create bridge: `oya incident bridge create --incident $INCIDENT_ID --channel #inc-governance-gates --severity sev2`.
3. Freeze risky automation: `oya flags set oya.governance.industry_baseline_refresh.incident_hold=true --cell $CELL --tenant $TENANT --reason $INCIDENT_ID`.
4. Enable circuit breaker: `oya ops breaker open governance-industry-baseline-refresh-circuit-breaker --cell $CELL --tenant $TENANT --ttl 30m --reason $INCIDENT_ID`.
5. Reduce blast radius: `kubectl -n governance scale deploy/governance-industry-baseline-refresh-worker --replicas=1`.
6. Protect tenant boundary: `oya tenancy quarantine --tenant $TENANT --reason governance-industry-baseline-refresh --ttl 60m`.
7. Pause promotion: incident hold PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
8. Drain queue safely: `oya ops governance industry-baseline-refresh drain --cell $CELL --tenant $TENANT --max-items 500 --dry-run`.
9. Execute bounded drain: `oya ops governance industry-baseline-refresh drain --cell $CELL --tenant $TENANT --max-items 500 --confirm $INCIDENT_ID`.
10. Replay missing audit events: `oya audit-chain replay --event-class EVT-GOVERNANCE-INDUSTRY_BASELINE_REFRESH-INCIDENT --incident $INCIDENT_ID --from evidence/incidents/$INCIDENT_ID.json`.
11. Rollback last deploy if causal: `kubectl -n governance rollout undo deploy/governance-industry-baseline-refresh-worker`.
12. Raise HPA cap if saturation: `kubectl -n governance patch hpa governance-industry-baseline-refresh-worker --type merge -p '{"spec":{"maxReplicas":12}}'`.
13. Throttle hot tenant: `oya ops rate-limit set --tenant $TENANT --surface governance.industry-baseline-refresh --rps 25 --ttl 30m`.
14. Block abusive principal: `oya identity principal suspend --principal suspected-abuse --tenant $TENANT --reason $INCIDENT_ID`.
15. Protect evidence: `oya evidence freeze --incident $INCIDENT_ID --paths microservices/governance/runbooks/industry-baseline-refresh.md,evidence/incidents/$INCIDENT_ID.json`.
16. Notify service owners: `oya notify service-owner --microservice governance --incident $INCIDENT_ID --channel #inc-governance-gates`.
17. Open external vendor ticket: `oya vendor ticket open --vendor primary-governance --incident $INCIDENT_ID --summary industry-baseline-refresh`.
18. Confirm breaker effect: `oya ops breaker status governance-industry-baseline-refresh-circuit-breaker --cell $CELL --tenant $TENANT --expect open`.
19. Confirm user impact reduced: `curl -s https://governance.internal.oyatie.dev/v1/governance/industry-baseline-refresh/incident-handoff/health -H "x-oya-tenant: $TENANT"`.
20. Emit mitigation audit: `oya audit-chain emit --event-class EVT-GOVERNANCE-INDUSTRY_BASELINE_REFRESH-INCIDENT --incident $INCIDENT_ID --field mitigation=active --field runbook=industry-baseline-refresh`.

### Mitigation Branch Guidance
- Branch A: backlog with healthy dependencies.
  - Required action: keep `governance-industry-baseline-refresh-circuit-breaker` open until `oya_governance_industry_baseline_refresh_error_ratio` is below 0.005 for 3 windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/governance-substrate/industry-baseline-refresh?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=109` to the incident.
  - Required audit: emit `EVT-GOVERNANCE-INDUSTRY_BASELINE_REFRESH-INCIDENT` with `branch=A`, `operator_id`, and `evidence_hash`.
- Branch B: downstream dependency degraded.
  - Required action: keep `governance-industry-baseline-refresh-circuit-breaker` open until `oya_governance_industry_baseline_refresh_error_ratio` is below 0.005 for 3 windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/governance-substrate/industry-baseline-refresh?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=110` to the incident.
  - Required audit: emit `EVT-GOVERNANCE-INDUSTRY_BASELINE_REFRESH-INCIDENT` with `branch=B`, `operator_id`, and `evidence_hash`.
- Branch C: bad deploy introduced regression.
  - Required action: keep `governance-industry-baseline-refresh-circuit-breaker` open until `oya_governance_industry_baseline_refresh_error_ratio` is below 0.005 for 3 windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/governance-substrate/industry-baseline-refresh?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=111` to the incident.
  - Required audit: emit `EVT-GOVERNANCE-INDUSTRY_BASELINE_REFRESH-INCIDENT` with `branch=C`, `operator_id`, and `evidence_hash`.
- Branch D: data replay or rebuild required.
  - Required action: keep `governance-industry-baseline-refresh-circuit-breaker` open until `oya_governance_industry_baseline_refresh_error_ratio` is below 0.005 for 3 windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/governance-substrate/industry-baseline-refresh?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=112` to the incident.
  - Required audit: emit `EVT-GOVERNANCE-INDUSTRY_BASELINE_REFRESH-INCIDENT` with `branch=D`, `operator_id`, and `evidence_hash`.

## Resolution Steps
1. Identify code owner path: `rg "industry_baseline_refresh|GovernanceIndustryBaselineRefreshCritical|governance.industry_baseline_refresh.incident_state" crates microservices/governance -g "!microservices/governance/runbooks/**"`.
2. Patch domain invariant: `edit oya-governance-domain where industry_baseline_refresh state transition is validated`.
3. Patch API guard: `edit microservices/governance/contracts/openapi.yaml or catalog REST binding if the failing path is north-south`.
4. Patch policy: `edit microservices/governance/policy/lane-execution.cedar or .md with explicit deny/permit branch`.
5. Patch runtime config: `edit microservices/governance/iac/k8s-deployment.yaml or secret-bindings.yaml if deploy/config drift caused the incident`.
6. Add regression test: `cargo test -p oya-governance-domain industry_baseline_refresh_incident_regression -- --nocapture`.
7. Add gate evidence: `cargo run -p oya-dev-cli -- gate validate governance-industry-baseline-refresh --fixture incident-industry-baseline-refresh.json`.
8. Add SLO assertion: `update microservices/governance/slos/* with alert GovernanceIndustryBaselineRefreshCritical when this was a missing alert`.
9. Add dashboard panel: `update microservices/governance/dashboards/lane-pass-rate.json with oya_governance_industry_baseline_refresh_error_ratio, oya_governance_industry_baseline_refresh_lag_seconds, and oya_governance_industry_baseline_refresh_queue_depth`.
10. Rebuild affected crate: `cargo check -p oya-governance-domain --all-targets`.
11. Run targeted tests: `cargo test -p oya-governance-domain --all-features`.
12. Run policy validation: `cargo run -p oya-dev-cli -- gate validate governance-policy --microservice governance`.
13. Deploy canary: `oya deploy canary --microservice governance --component industry-baseline-refresh-worker --cell $CELL --weight 1`.
14. Watch burn rate: `oya ops watch --metric oya_governance_industry_baseline_refresh_error_ratio --threshold 0.005 --window 30m --cell $CELL`.
15. Close circuit breaker: `oya ops breaker close governance-industry-baseline-refresh-circuit-breaker --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
16. Unfreeze automation: `oya flags set oya.governance.industry_baseline_refresh.incident_hold=false --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
17. Resume promotion: recovery PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
18. Seal resolution audit: `oya audit-chain emit --event-class EVT-GOVERNANCE-INDUSTRY_BASELINE_REFRESH-INCIDENT --incident $INCIDENT_ID --field resolution=complete --field runbook=industry-baseline-refresh`.
19. Verify seal: `oya audit-chain verify --event-class EVT-GOVERNANCE-INDUSTRY_BASELINE_REFRESH-INCIDENT --incident $INCIDENT_ID`.
20. Attach final evidence: `oya evidence attach --incident $INCIDENT_ID --file evidence/incidents/$INCIDENT_ID.json --kind final-resolution`.

### Code Paths To Inspect First
- `oya-governance-domain`: inspect for industry_baseline_refresh invariants, alert emission, and ADR-0263 evidence fields before touching adjacent code path 1.
- `oya-dev-cli`: inspect for industry_baseline_refresh invariants, alert emission, and ADR-0263 evidence fields before touching adjacent code path 2.
- `oya-policy-cedar-domain`: inspect for industry_baseline_refresh invariants, alert emission, and ADR-0263 evidence fields before touching adjacent code path 3.
- `microservices/governance/contracts/`: verify this surface only when the incident evidence points there.
- `microservices/governance/dashboards/lane-pass-rate.json`: verify this surface only when the incident evidence points there.
- `microservices/governance/slos/`: verify this surface only when the incident evidence points there.
- `microservices/governance/policy/lane-execution.*`: verify this surface only when the incident evidence points there.

## Verification Checklist
- GovernanceIndustryBaselineRefreshCritical and GovernanceIndustryBaselineRefreshSloBurn are both resolved in Alertmanager for 30 minutes.
- oya_governance_industry_baseline_refresh_error_ratio < 0.005 for 3 consecutive 10 minute windows.
- oya_governance_industry_baseline_refresh_lag_seconds < 120 for all production cells.
- oya_governance_industry_baseline_refresh_queue_depth is draining and not growing for the affected tenant.
- dashboard https://grafana.dev.oyatie.internal/d/governance-substrate/industry-baseline-refresh?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=109 shows green panels for the affected cell.
- audit-chain query for EVT-GOVERNANCE-INDUSTRY_BASELINE_REFRESH-INCIDENT returns mitigation and resolution events.
- circuit breaker governance-industry-baseline-refresh-circuit-breaker is closed after rollback window.
- feature flag oya.governance.industry_baseline_refresh.incident_hold is false for the affected tenant unless long-term hold is approved.
- runbook invocation evidence is attached to evidence/incidents/$INCIDENT_ID.json.
- service owner acknowledged final handoff in #inc-governance-gates.

## Postmortem Template
Use this exact skeleton for the incident document. The field names are intentionally stable for ADR-0263 audit emission extraction.
```markdown
---
doc_class: IncidentPostmortem
runbook_id: governance-industry-baseline-refresh
microservice: governance
event_class: EVT-GOVERNANCE-INDUSTRY_BASELINE_REFRESH-INCIDENT
incident_id: <INC-...>
severity: sev2
status: draft
detected_at: <UTC>
mitigated_at: <UTC>
resolved_at: <UTC>
commander: <handle>
evidence_hash: <sha256>
---

# Industry Baseline Refresh postmortem

## Summary
- What happened in governance/industry-baseline-refresh.
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
- Emit EVT-GOVERNANCE-INDUSTRY_BASELINE_REFRESH-INCIDENT with incident_id, tenant_id, cell_id, principal_id, decision_id, evidence_hash, operator_id, runbook_id.
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
- Customer communications: use status page component `oyatie-governance-industry-baseline-refresh` and keep private details in the incident channel.
- Regulatory clock: if any tenant data exposure is possible, start the compliance 72h assessment timer even if exposure is unconfirmed.
- Executive notice: sev0 or fleet-wide sev1 goes to `#exec-incident-readout` within 30 minutes.

## Cross-µservice Coordination
- Notify `identity`: `oya incident handoff --target identity --source governance --runbook industry-baseline-refresh --incident $INCIDENT_ID --severity sev2 --branch A`; expect `202 accepted`.
- Require `identity` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `tenancy`: `oya incident handoff --target tenancy --source governance --runbook industry-baseline-refresh --incident $INCIDENT_ID --severity sev2 --branch B`; expect `202 accepted`.
- Require `tenancy` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `compliance`: `oya incident handoff --target compliance --source governance --runbook industry-baseline-refresh --incident $INCIDENT_ID --severity sev2 --branch C`; expect `202 accepted`.
- Require `compliance` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `observability`: `oya incident handoff --target observability --source governance --runbook industry-baseline-refresh --incident $INCIDENT_ID --severity sev2 --branch D`; expect `202 accepted`.
- Require `observability` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Observability handoff API: `oya incident handoff --target observability --source governance --runbook industry-baseline-refresh --incident $INCIDENT_ID`.
- Governance handoff API: `oya incident handoff --target governance --source governance --runbook industry-baseline-refresh --incident $INCIDENT_ID`.
- Compliance handoff API: `oya incident handoff --target compliance --source governance --runbook industry-baseline-refresh --incident $INCIDENT_ID`.
- Identity handoff API: `oya incident handoff --target identity --source governance --runbook industry-baseline-refresh --incident $INCIDENT_ID`.
- Tenancy handoff API: `oya incident handoff --target tenancy --source governance --runbook industry-baseline-refresh --incident $INCIDENT_ID`.

## Handoff Notes
- Do not hand off with only the alert name; include oya_governance_industry_baseline_refresh_error_ratio, oya_governance_industry_baseline_refresh_lag_seconds, oya_governance_industry_baseline_refresh_queue_depth, current breaker state, and audit seal status.
- Keep governance-industry-baseline-refresh-circuit-breaker owner as axis-foundry + council-architecture + ops-sre-reliability until the receiving service explicitly accepts.
- If another runbook owns the downstream fix, link this incident as upstream and keep this runbook open until downstream verification returns green.
- Close only after EVT-GOVERNANCE-INDUSTRY_BASELINE_REFRESH-INCIDENT has a sealed resolution row and every coordination endpoint above has either accepted or explicitly declined scope.

## Sources Checked During This Substance Pass
- `microservices/governance/dashboards/` for dashboard names and operational panels.
- `microservices/governance/slos/` for OpenSLO alert vocabulary and threshold alignment.
- `microservices/governance/policy/` for named policy and authorization surfaces.
- `microservices/governance/catalog/` for component and owner vocabulary.
- Existing thin runbook topic `industry-baseline-refresh` was preserved as the scenario anchor while replacing generic steps with concrete commands.
