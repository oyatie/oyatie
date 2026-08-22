---
doc_class: Runbook
title: Focus Export Failure
status: Accepted
date: 2026-05-20
microservice: finops-portal
severity: sev1
audience: sre
owner_team: axis-finops + ops-sre-reliability + finance-operations
source_wave: codex-runbooks-substrate-w2
change_scope: substance rewrite of existing thin runbook
doc_status: published
---

# Runbook: Focus Export Failure

## Operator Contract
- Runbook id: finops-portal-focus-export-failure.
- Primary service namespace: `finops-portal`.
- Owning rotation: PagerDuty finops-primary; finance-ops secondary.
- Incident channel: `#inc-finops-portal`.
- Operational focus: protecting tenant cost truth, invoice correctness, FOCUS export, budget alerts, and recommendation evidence while resolving focus export failure.
- External dependencies: AWS Enterprise Support billing desk; GCP Cloud Billing support; Azure Cost Management support; FinOps Foundation FOCUS working group.
- API authority: `https://finops-portal.internal.oyatie.dev/v1/finops/focus-export-failure/incident-handoff`.
- Audit event class: `EVT_FINOPS_PORTAL_FOCUS_EXPORT_FAILURE_INCIDENT` with ADR-0263 fields `incident_id`, `tenant_id`, `cell_id`, `microservice`, `runbook_id`, `decision_id`, `evidence_hash`, `operator_id`.
- Stop condition: mitigation has held for 30 minutes, `FinopsPortalFocusExportFailureCritical` is green, and every handoff API in Cross-microservice Coordination returns `202 accepted`.
- Safety invariant: never clear the incident until `EVT_FINOPS_PORTAL_FOCUS_EXPORT_FAILURE_INCIDENT` is sealed and the postmortem skeleton exists under `evidence/postmortems/finops-portal-focus-export-failure-<incident-id>.md`.

## Trigger Conditions
- Page on alert `FinopsPortalFocusExportFailureCritical` when `finops_portal_focus_export_failure_error_ratio > 0.02` for 10 minutes in any production cell.
- Page on alert `FinopsPortalFocusExportFailureSloBurn` when `finops_portal_focus_export_failure_lag_seconds > 300` for 2 consecutive evaluator windows.
- Open a sev1 if `finops_portal_focus_export_failure_correctness_ratio < 0.9999` and the affected label set includes `tenant_id`, `cell_id`, or `principal_id`.
- Open a sev1 if `finops_portal_focus_export_failure_queue_depth > 5000` for 15 minutes or retry backlog grows by more than 20 percent in one 5 minute window.
- Trigger from customer report when Support tags the case `finops-portal.focus-export-failure.customer_visible` in Zendesk.
- Trigger from CI when `cargo run -p dev-cli -- gate validate finops-portal-focus-export-failure --production-snapshot` exits non-zero against the latest production evidence bundle.
- Primary dashboard: `https://grafana.dev.oyatie.internal/d/finops-portal-substrate/focus-export-failure?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=116`.
- Secondary dashboard: `https://grafana.dev.oyatie.internal/d/finops-portal-substrate/focus-export-failure?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=213`.
- Loki explorer: `https://grafana.dev.oyatie.internal/explore?query={namespace="finops-portal",runbook="focus-export-failure"}`.
- Alertmanager route: `oyatie-finops-portal-focus-export-failure-critical`; silence only with incident commander approval and `EVT_FINOPS_PORTAL_FOCUS_EXPORT_FAILURE_INCIDENT` evidence.
- Synthetic probe: `oya ops probe finops-portal focus-export-failure --cell prod-us-east-1 --tenant synthetic-canary` returns `healthy=true`.
- Drift detector: `registry/finops-portal/focus-export-failure/expected-state.json` hash differs from live `https://finops-portal.internal.oyatie.dev/v1/finops/admin/state-hash`.
- Service-specific metric `finops_portal_focus_export_failure_allocation_delta_cents` exceeds the threshold documented in `microservices/finops-portal/slos/tenant-invoice-render-latency.openslo.yaml`.

## Symptoms
- User-facing impact: tenant invoices, cost drilldowns, budget alerts, regulator evidence, and reservation recommendations may be stale or wrong.
- Operators see Grafana panel `tenant-cost-drilldown.grafana.json / Focus Export Failure burn rate` turn red before the primary alert resolves.
- Loki signature `finops_portal.focus_export_failure.incident_state=failed` appears with fields `incident_id`, `tenant_id`, `cell_id`, `decision_id`, `evidence_hash`.
- Kubernetes events include `reason=FinopsPortalFocusExportFailureDegraded` on deployment `finops-portal-focus-export-failure-worker` or `finops-portal`.
- Audit-chain shows missing or delayed `EVT_FINOPS_PORTAL_FOCUS_EXPORT_FAILURE_INCIDENT` entries when queried with `oya audit-chain query --event-class EVT_FINOPS_PORTAL_FOCUS_EXPORT_FAILURE_INCIDENT --since 30m`.
- Metric pattern: `finops_portal_focus_export_failure_error_ratio` rises before `finops_portal_focus_export_failure_lag_seconds`; if lag rises first, suspect dependency saturation rather than local regression.
- Metric pattern: `finops_portal_focus_export_failure_queue_depth` increases while pod CPU stays below 40 percent; suspect downstream refusal, replay backlog, or feature flag deadlock.
- Tenant-specific shape: one `tenant_id` dominates labels in `finops_portal_focus_export_failure_queue_depth`; isolate tenant before fleet mitigation.
- Fleet-wide shape: at least three cells report `FinopsPortalFocusExportFailureCritical` in one 15 minute window; switch to cross-cell bridge even if individual tenants are low-volume.
- Log signature `decision=deny reason=focus-export-failure.policy_guard` means the guard is working; investigate caller inputs before rollback.
- Log signature `decision=permit reason=focus-export-failure.break_glass` means manual intervention is active; confirm two-person authorization.
- Log signature `audit_emit_status=stalled event_class=EVT_FINOPS_PORTAL_FOCUS_EXPORT_FAILURE_INCIDENT` means mitigation cannot be closed until replay succeeds.
- Service-specific metric pattern: `finops_portal_focus_export_failure_invoice_render_lag_seconds` rises while `finops_portal_focus_export_failure_recommendation_staleness_seconds` is flat; inspect local worker health before escalating vendors.
- Service-specific metric pattern: `finops_portal_focus_export_failure_recommendation_staleness_seconds` rises while `finops_portal_focus_export_failure_error_ratio` is flat; suspect stale export, stale recommendation, stale projection, or vendor dependency lag.

## Diagnostic Steps
1. Set incident variables: `export INCIDENT_ID=INC-finops-portal-focus-export-failure-$(date -u +%Y%m%dT%H%M%SZ); export CELL=prod-us-east-1; export TENANT=synthetic-canary`.
2. Confirm active alerts: `curl -s https://finops-portal.internal.oyatie.dev/v1/finops/alerts?runbook=focus-export-failure | jq .alerts`.
3. Check Kubernetes rollout: `kubectl -n finops-portal rollout status deploy/finops-portal-focus-export-failure-worker --timeout=60s`.
4. List unhealthy pods: `kubectl -n finops-portal get pods -l app=focus-export-failure -o wide`.
5. Read structured logs: `kubectl -n finops-portal logs deploy/finops-portal-focus-export-failure-worker --since=30m | rg "finops_portal.focus_export_failure.incident_state|FinopsPortalFocusExportFailureCritical|EVT_FINOPS_PORTAL_FOCUS_EXPORT_FAILURE_INCIDENT"`.
6. Query Loki directly: `logcli query '{namespace="finops-portal",runbook="focus-export-failure"}' --since=30m --limit=200`.
7. Check Prometheus error ratio: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=finops_portal_focus_export_failure_error_ratio{cell="prod-us-east-1"}'`.
8. Check lag: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=finops_portal_focus_export_failure_lag_seconds{cell="prod-us-east-1"}'`.
9. Check queue: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=finops_portal_focus_export_failure_queue_depth{cell="prod-us-east-1"}'`.
10. Check service-specific signal: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=finops_portal_focus_export_failure_allocation_delta_cents{cell="prod-us-east-1"}'`.
11. Open primary dashboard: `open "https://grafana.dev.oyatie.internal/d/finops-portal-substrate/focus-export-failure?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=116&var-incident=$INCIDENT_ID"`.
12. Open secondary dashboard: `open "https://grafana.dev.oyatie.internal/d/finops-portal-substrate/focus-export-failure?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=213&var-tenant=$TENANT"`.
13. Verify audit-chain emission: `oya audit-chain query --event-class EVT_FINOPS_PORTAL_FOCUS_EXPORT_FAILURE_INCIDENT --since 30m --cell $CELL --tenant $TENANT`.
14. Verify service state: `oya ops finops-portal focus-export-failure status --cell $CELL --tenant $TENANT --output json`.
15. Run production snapshot gate: `cargo run -p dev-cli -- gate validate finops-portal-focus-export-failure --production-snapshot --cell $CELL`.
16. Run crate smoke test: `cargo test -p cloud-finops-domain focus_export_failure -- --nocapture`.
17. Check API contract smoke: `curl -s https://finops-portal.internal.oyatie.dev/v1/finops/focus-export-failure/incident-handoff -H "x-tenant: $TENANT"`.
18. Inspect config: `test -f microservices/finops-portal/iac/helm/finops-portal/values.yaml && sed -n '1,180p' microservices/finops-portal/iac/helm/finops-portal/values.yaml`.
19. Inspect feature flags: `oya flags get oya.finops-portal.focus_export_failure.incident_hold --cell $CELL --tenant $TENANT --output yaml`.
20. Inspect circuit breaker: `oya ops breaker status finops-portal-focus-export-failure-circuit-breaker --cell $CELL --tenant $TENANT`.
21. Check recent deploy: `kubectl -n finops-portal rollout history deploy/finops-portal-focus-export-failure-worker | tail -20`.
22. Check policy file: `test -f microservices/finops-portal/policy/cedar/ops-finops-dashboard-access.cedar || find microservices/finops-portal/policy -maxdepth 2 -type f | sort`.
23. Check SLO files: `ls microservices/finops-portal/slos/*.openslo.yaml | sort | rg "tenant|focus"`.
24. Check catalog components: `find microservices/finops-portal/catalog -maxdepth 1 -type f | sort | rg "budget|forecasting|showback|chargeback|rightsizing|commitment"`.
25. Run targeted SQL state query: `psql $OYATIE_PROD_DSN -c "select incident_id, tenant_id, cell_id, state, updated_at from finops_portal_focus_export_failure_incidents where updated_at > now() - interval '30 minutes' order by updated_at desc limit 20;"`.
26. Confirm no cross-cell spread: `oya ops cells query --metric finops_portal_focus_export_failure_error_ratio --window 30m --threshold 0.02`.
27. Snapshot evidence: `oya evidence snapshot --incident $INCIDENT_ID --microservice finops-portal --runbook focus-export-failure --output evidence/incidents/$INCIDENT_ID.json`.

### Diagnostic Decision Tree
```text
Focus Export Failure incident decision tree
1. Is FinopsPortalFocusExportFailureCritical firing in more than one cell?
   |-- yes: declare fleet incident, page PagerDuty finops-primary; finance-ops secondary, and run cross-cell containment.
   |-- no: keep scope to the affected cell and continue tenant isolation checks.
2. Does finops_portal_focus_export_failure_queue_depth grow while finops_portal_focus_export_failure_error_ratio is flat?
   |-- yes: downstream dependency, replay backlog, or queue-drain issue; choose mitigation branch B.
   |-- no: local regression, bad input, or policy/config drift; continue branch selection.
3. Does audit-chain show EVT_FINOPS_PORTAL_FOCUS_EXPORT_FAILURE_INCIDENT gaps?
   |-- yes: do not close; run evidence replay before resolution.
   |-- no: mitigation can proceed after state is green.
4. Is customer, finance, security, or regulator impact confirmed?
   |-- yes: promote severity, open #inc-finops-portal, and notify compliance or security handoff.
   |-- no: keep internal incident and collect evidence.
```
- Branch A (tenant bill correctness risk): use the matching mitigation block below and record `decision_branch=A` in `EVT_FINOPS_PORTAL_FOCUS_EXPORT_FAILURE_INCIDENT`.
- Branch B (export or dashboard freshness degradation): use the matching mitigation block below and record `decision_branch=B` in `EVT_FINOPS_PORTAL_FOCUS_EXPORT_FAILURE_INCIDENT`.
- Branch C (recommendation engine dependency degraded): use the matching mitigation block below and record `decision_branch=C` in `EVT_FINOPS_PORTAL_FOCUS_EXPORT_FAILURE_INCIDENT`.
- Branch D (regulator or finance close impact): use the matching mitigation block below and record `decision_branch=D` in `EVT_FINOPS_PORTAL_FOCUS_EXPORT_FAILURE_INCIDENT`.

## Mitigation Steps
1. Acknowledge page: `pd incident ack --service finops-portal --incident $INCIDENT_ID`.
2. Create bridge: `oya incident bridge create --incident $INCIDENT_ID --channel #inc-finops-portal --severity sev1`.
3. Freeze risky automation: `oya flags set oya.finops-portal.focus_export_failure.incident_hold=true --cell $CELL --tenant $TENANT --reason $INCIDENT_ID`.
4. Enable circuit breaker: `oya ops breaker open finops-portal-focus-export-failure-circuit-breaker --cell $CELL --tenant $TENANT --ttl 30m --reason $INCIDENT_ID`.
5. Reduce blast radius: `kubectl -n finops-portal scale deploy/finops-portal-focus-export-failure-worker --replicas=1`.
6. Protect tenant boundary: `oya tenancy quarantine --tenant $TENANT --reason finops-portal-focus-export-failure --ttl 60m`.
7. Pause promotion: incident hold PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
8. Drain queue safely: `oya ops finops-portal focus-export-failure drain --cell $CELL --tenant $TENANT --max-items 500 --dry-run`.
9. Execute bounded drain: `oya ops finops-portal focus-export-failure drain --cell $CELL --tenant $TENANT --max-items 500 --confirm $INCIDENT_ID`.
10. Replay missing audit events: `oya audit-chain replay --event-class EVT_FINOPS_PORTAL_FOCUS_EXPORT_FAILURE_INCIDENT --incident $INCIDENT_ID --from evidence/incidents/$INCIDENT_ID.json`.
11. Rollback last deploy if causal: `kubectl -n finops-portal rollout undo deploy/finops-portal-focus-export-failure-worker`.
12. Raise HPA cap if saturation is proven: `kubectl -n finops-portal patch hpa finops-portal-focus-export-failure-worker --type merge -p '{"spec":{"maxReplicas":12}}'`.
13. Throttle hot tenant: `oya ops rate-limit set --tenant $TENANT --surface finops-portal.focus-export-failure --rps 25 --ttl 30m`.
14. Block abusive principal when relevant: `oya identity principal suspend --principal suspected-abuse --tenant $TENANT --reason $INCIDENT_ID`.
15. Protect evidence: `oya evidence freeze --incident $INCIDENT_ID --paths microservices/finops-portal/runbooks/focus-export-failure.md,evidence/incidents/$INCIDENT_ID.json`.
16. Notify service owners: `oya notify service-owner --microservice finops-portal --incident $INCIDENT_ID --channel #inc-finops-portal`.
17. Open external vendor ticket: `oya vendor ticket open --vendor "AWS Enterprise Support billing desk" --incident $INCIDENT_ID --summary finops-portal-focus-export-failure`.
18. Confirm breaker effect: `oya ops breaker status finops-portal-focus-export-failure-circuit-breaker --cell $CELL --tenant $TENANT --expect open`.
19. Confirm user impact reduced: `curl -s https://finops-portal.internal.oyatie.dev/v1/finops/focus-export-failure/incident-handoff/health -H "x-tenant: $TENANT"`.
20. Emit mitigation audit: `oya audit-chain emit --event-class EVT_FINOPS_PORTAL_FOCUS_EXPORT_FAILURE_INCIDENT --incident $INCIDENT_ID --field mitigation=active --field runbook=focus-export-failure`.

### Mitigation Branch Guidance
- Branch A: tenant bill correctness risk.
  - Required action: keep `finops-portal-focus-export-failure-circuit-breaker` open until `finops_portal_focus_export_failure_error_ratio` is below 0.005 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/finops-portal-substrate/focus-export-failure?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=117` to the incident.
  - Required audit: emit `EVT_FINOPS_PORTAL_FOCUS_EXPORT_FAILURE_INCIDENT` with `branch=A`, `operator_id`, and `evidence_hash`.
- Branch B: export or dashboard freshness degradation.
  - Required action: keep `finops-portal-focus-export-failure-circuit-breaker` open until `finops_portal_focus_export_failure_error_ratio` is below 0.01 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/finops-portal-substrate/focus-export-failure?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=118` to the incident.
  - Required audit: emit `EVT_FINOPS_PORTAL_FOCUS_EXPORT_FAILURE_INCIDENT` with `branch=B`, `operator_id`, and `evidence_hash`.
- Branch C: recommendation engine dependency degraded.
  - Required action: keep `finops-portal-focus-export-failure-circuit-breaker` open until `finops_portal_focus_export_failure_error_ratio` is below 0.005 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/finops-portal-substrate/focus-export-failure?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=119` to the incident.
  - Required audit: emit `EVT_FINOPS_PORTAL_FOCUS_EXPORT_FAILURE_INCIDENT` with `branch=C`, `operator_id`, and `evidence_hash`.
- Branch D: regulator or finance close impact.
  - Required action: keep `finops-portal-focus-export-failure-circuit-breaker` open until `finops_portal_focus_export_failure_error_ratio` is below 0.01 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/finops-portal-substrate/focus-export-failure?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=120` to the incident.
  - Required audit: emit `EVT_FINOPS_PORTAL_FOCUS_EXPORT_FAILURE_INCIDENT` with `branch=D`, `operator_id`, and `evidence_hash`.

## Resolution Steps
1. Identify code owner path: `rg "focus_export_failure|FinopsPortalFocusExportFailureCritical|finops_portal.focus_export_failure.incident_state" crates microservices/finops-portal -g "!microservices/finops-portal/runbooks/**"`.
2. Patch domain invariant: `edit cloud-finops-domain where focus_export_failure state transition is validated`.
3. Patch API guard: `edit microservices/finops-portal/contracts/tenant-invoice-public.openapi.yaml if the failing path is north-south or async handoff`.
4. Patch policy: `edit microservices/finops-portal/policy/cedar/ops-finops-dashboard-access.cedar with explicit deny/permit branch and tenant/cell scope`.
5. Patch runtime config: `edit microservices/finops-portal/iac/helm/finops-portal/values.yaml if deploy/config drift caused the incident`.
6. Add regression test: `cargo test -p cloud-finops-domain focus_export_failure_incident_regression -- --nocapture`.
7. Add gate evidence: `cargo run -p dev-cli -- gate validate finops-portal-focus-export-failure --fixture incident-focus-export-failure.json`.
8. Add SLO assertion: `update microservices/finops-portal/slos/tenant-invoice-render-latency.openslo.yaml with alert FinopsPortalFocusExportFailureCritical when this was a missing alert`.
9. Add dashboard panel: `update microservices/finops-portal/dashboards/tenant-cost-drilldown.grafana.json with finops_portal_focus_export_failure_error_ratio, finops_portal_focus_export_failure_lag_seconds, and finops_portal_focus_export_failure_queue_depth`.
10. Rebuild affected crate: `cargo check -p cloud-finops-domain --all-targets`.
11. Run targeted tests: `cargo test -p cloud-finops-domain --all-features`.
12. Run policy validation: `cargo run -p dev-cli -- gate validate finops-portal-policy --microservice finops-portal`.
13. Deploy canary: `oya deploy canary --microservice finops-portal --component finops-portal-focus-export-failure-worker --cell $CELL --weight 1`.
14. Watch burn rate: `oya ops watch --metric finops_portal_focus_export_failure_error_ratio --threshold 0.005 --window 30m --cell $CELL`.
15. Close circuit breaker: `oya ops breaker close finops-portal-focus-export-failure-circuit-breaker --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
16. Unfreeze automation: `oya flags set oya.finops-portal.focus_export_failure.incident_hold=false --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
17. Resume promotion: recovery PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
18. Seal resolution audit: `oya audit-chain emit --event-class EVT_FINOPS_PORTAL_FOCUS_EXPORT_FAILURE_INCIDENT --incident $INCIDENT_ID --field resolution=complete --field runbook=focus-export-failure`.
19. Verify seal: `oya audit-chain verify --event-class EVT_FINOPS_PORTAL_FOCUS_EXPORT_FAILURE_INCIDENT --incident $INCIDENT_ID`.
20. Attach final evidence: `oya evidence attach --incident $INCIDENT_ID --file evidence/incidents/$INCIDENT_ID.json --kind final-resolution`.

### Code Paths To Inspect First
- `cloud-finops-domain`: inspect for `focus_export_failure` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `cloud-finops-kernel`: inspect for `focus_export_failure` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `cloud-finops-api`: inspect for `focus_export_failure` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `cloud-billing-kernel`: inspect for `focus_export_failure` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `microservices/finops-portal/contracts/tenant-invoice-public.openapi.yaml`: verify request/response or event contract only when incident evidence points there.
- `microservices/finops-portal/contracts/focus-export-internal.asyncapi.yaml`: verify request/response or event contract only when incident evidence points there.
- `microservices/finops-portal/contracts/cost-allocation-policy-internal.proto`: verify request/response or event contract only when incident evidence points there.
- `microservices/finops-portal/dashboards/tenant-cost-drilldown.grafana.json`: verify panel coverage for `finops_portal_focus_export_failure_error_ratio`, `finops_portal_focus_export_failure_lag_seconds`, and `finops_portal_focus_export_failure_allocation_delta_cents`.
- `microservices/finops-portal/slos/`: verify alert vocabulary and threshold alignment before changing runtime thresholds.
- `microservices/finops-portal/policy/`: verify policy branch ownership before relaxing deny rules or emergency bypasses.

## Verification Checklist
- `FinopsPortalFocusExportFailureCritical` and `FinopsPortalFocusExportFailureSloBurn` are both resolved in Alertmanager for 30 minutes.
- `finops_portal_focus_export_failure_error_ratio < 0.005` for 3 consecutive 10 minute windows.
- `finops_portal_focus_export_failure_lag_seconds < 120` for all production cells.
- `finops_portal_focus_export_failure_queue_depth` is draining and not growing for the affected tenant.
- Service-specific signal `finops_portal_focus_export_failure_allocation_delta_cents` is below the threshold documented in `microservices/finops-portal/slos/tenant-invoice-render-latency.openslo.yaml`.
- dashboard `https://grafana.dev.oyatie.internal/d/finops-portal-substrate/focus-export-failure?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=116` shows green panels for the affected cell.
- audit-chain query for `EVT_FINOPS_PORTAL_FOCUS_EXPORT_FAILURE_INCIDENT` returns mitigation and resolution events.
- circuit breaker `finops-portal-focus-export-failure-circuit-breaker` is closed after rollback window.
- feature flag `oya.finops-portal.focus_export_failure.incident_hold` is false for the affected tenant unless long-term hold is approved.
- runbook invocation evidence is attached to `evidence/incidents/$INCIDENT_ID.json`.
- service owner acknowledged final handoff in `#inc-finops-portal`.

## Postmortem Template
Use this exact skeleton for the incident document. The field names are intentionally stable for ADR-0263 audit emission extraction.
```markdown
---
doc_class: IncidentPostmortem
runbook_id: finops-portal-focus-export-failure
microservice: finops-portal
event_class: EVT_FINOPS_PORTAL_FOCUS_EXPORT_FAILURE_INCIDENT
incident_id: <INC-...>
severity: sev1
status: draft
detected_at: <UTC>
mitigated_at: <UTC>
resolved_at: <UTC>
commander: <handle>
evidence_hash: <sha256>
---

# Focus Export Failure postmortem

## Summary
- What happened in finops-portal/focus-export-failure.
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
- Emit EVT_FINOPS_PORTAL_FOCUS_EXPORT_FAILURE_INCIDENT with incident_id, tenant_id, cell_id, principal_id, decision_id, evidence_hash, operator_id, runbook_id.
- Attach dashboard snapshot URLs and command transcripts.
- Seal mitigation and resolution events before closure.

## Corrective Actions
- Action, owner, due date, validation command, linked issue.
```

## Escalation Path
- Primary on-call: PagerDuty finops-primary; finance-ops secondary.
- Incident SLA: ack 3m for sev0/sev1, 10m for sev2, 30m for sev3; status update every 10m until `FinopsPortalFocusExportFailureCritical` clears.
- Incident commander: first responder from axis-finops + ops-sre-reliability + finance-operations; transfer only by explicit message in `#inc-finops-portal`.
- Security escalation: page `ops-security-primary` immediately for sev0, credential, cross-tenant, fraud, or audit-seal symptoms.
- Compliance escalation: page `dpo-office-duty` when tenant data, regulator evidence, money movement, or breach-clock symptoms are present.
- Architecture escalation: page `council-architecture-reviewer` before manual bypass, policy rollback, or invariant relaxation.
- External vendors: AWS Enterprise Support billing desk; GCP Cloud Billing support; Azure Cost Management support; FinOps Foundation FOCUS working group. Open a ticket once local dependency health is proven and vendor dependency remains suspect.
- Customer communications: use status page component `oyatie-finops-portal-focus-export-failure` and keep private details in the incident channel.
- Regulatory clock: if tenant data, financial correctness, or evidence integrity is possibly affected, start the compliance 72h assessment timer even if exposure is unconfirmed.
- Executive notice: sev0 or fleet-wide sev1 goes to `#exec-incident-readout` within 30 minutes.

## Cross-µservice Coordination
- Notify `payments`: `oya incident handoff --target payments --source finops-portal --runbook focus-export-failure --incident $INCIDENT_ID --severity sev1 --branch A`; expect `202 accepted`.
- Require `payments` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `metering`: `oya incident handoff --target metering --source finops-portal --runbook focus-export-failure --incident $INCIDENT_ID --severity sev1 --branch B`; expect `202 accepted`.
- Require `metering` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `audit-chain`: `oya incident handoff --target audit-chain --source finops-portal --runbook focus-export-failure --incident $INCIDENT_ID --severity sev1 --branch C`; expect `202 accepted`.
- Require `audit-chain` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `compliance`: `oya incident handoff --target compliance --source finops-portal --runbook focus-export-failure --incident $INCIDENT_ID --severity sev1 --branch D`; expect `202 accepted`.
- Require `compliance` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Observability handoff API: `oya incident handoff --target observability --source finops-portal --runbook focus-export-failure --incident $INCIDENT_ID`.
- Governance handoff API: `oya incident handoff --target governance --source finops-portal --runbook focus-export-failure --incident $INCIDENT_ID`.
- Compliance handoff API: `oya incident handoff --target compliance --source finops-portal --runbook focus-export-failure --incident $INCIDENT_ID`.
- Audit-chain handoff API: `oya incident handoff --target audit-chain --source finops-portal --runbook focus-export-failure --incident $INCIDENT_ID`.
- Tenancy handoff API: `oya incident handoff --target tenancy --source finops-portal --runbook focus-export-failure --incident $INCIDENT_ID`.

## Handoff Notes
- Do not hand off with only the alert name; include `finops_portal_focus_export_failure_error_ratio`, `finops_portal_focus_export_failure_lag_seconds`, `finops_portal_focus_export_failure_queue_depth`, `finops_portal_focus_export_failure_allocation_delta_cents`, current breaker state, and audit seal status.
- Keep `finops-portal-focus-export-failure-circuit-breaker` owner as axis-finops + ops-sre-reliability + finance-operations until the receiving service explicitly accepts.
- If another runbook owns the downstream fix, link this incident as upstream and keep this runbook open until downstream verification returns green.
- Close only after `EVT_FINOPS_PORTAL_FOCUS_EXPORT_FAILURE_INCIDENT` has a sealed resolution row and every coordination endpoint above has either accepted or explicitly declined scope.

## Sources Checked During This Substance Pass
- `microservices/finops-portal/dashboards/` for dashboard names and operational panels: tenant-cost-drilldown.grafana.json, budget-alerts.grafana.json, rightsizing-recommendations.grafana.json, anomaly-investigation.grafana.json.
- `microservices/finops-portal/slos/` for OpenSLO alert vocabulary and threshold alignment: tenant-invoice-render-latency.openslo.yaml, focus-export-availability.openslo.yaml, cost-allocation-policy-change-latency.openslo.yaml, anomaly-explanation-latency.openslo.yaml.
- `microservices/finops-portal/policy/` for named policy and authorization surfaces: policy/cedar/ops-finops-dashboard-access.cedar, policy/cedar/tenant-isolation.cedar, policy/cedar/regulator-evidence-emit.cedar.
- `microservices/finops-portal/contracts/` for API, AsyncAPI, proto, and adapter surfaces: contracts/tenant-invoice-public.openapi.yaml, contracts/focus-export-internal.asyncapi.yaml, contracts/cost-allocation-policy-internal.proto.
- `microservices/finops-portal/catalog/` for component and owner vocabulary; existing runbook topic `focus-export-failure` was preserved as the scenario anchor.
