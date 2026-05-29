---
doc_class: Runbook
title: Ontology Read Library Fallback
status: Accepted
date: 2026-05-20
microservice: ontology
severity: sev2
audience: oncall-engineer
owner_team: axis-ontology + ops-sre-reliability + data-boundary-security
source_wave: codex-runbooks-substrate-w2
change_scope: substance rewrite of existing thin runbook
doc_status: published
---

# Runbook: Ontology Read Library Fallback

## Operator Contract
- Runbook id: ontology-ontology-read-library-fallback.
- Primary service namespace: `ontology`.
- Owning rotation: PagerDuty oya-ontology-primary; graph-platform secondary.
- Incident channel: `#inc-ontology`.
- Operational focus: protecting object graph isolation, entity projection correctness, read-path freshness, and graph query safety while resolving ontology read library fallback.
- External dependencies: ClickHouse support; Oracle PostgreSQL support; Cedar policy runtime support.
- API authority: `https://ontology.internal.oyatie.dev/v1/ontology/ontology-read-library-fallback/incident-handoff`.
- Audit event class: `EVT_ONTOLOGY_ONTOLOGY_READ_LIBRARY_FALLBACK_INCIDENT` with ADR-0263 fields `incident_id`, `tenant_id`, `cell_id`, `microservice`, `runbook_id`, `decision_id`, `evidence_hash`, `operator_id`.
- Stop condition: mitigation has held for 30 minutes, `OntologyOntologyReadLibraryFallbackCritical` is green, and every handoff API in Cross-microservice Coordination returns `202 accepted`.
- Safety invariant: never clear the incident until `EVT_ONTOLOGY_ONTOLOGY_READ_LIBRARY_FALLBACK_INCIDENT` is sealed and the postmortem skeleton exists under `evidence/postmortems/ontology-ontology-read-library-fallback-<incident-id>.md`.

## Trigger Conditions
- Page on alert `OntologyOntologyReadLibraryFallbackCritical` when `oya_ontology_ontology_read_library_fallback_error_ratio > 0.02` for 10 minutes in any production cell.
- Page on alert `OntologyOntologyReadLibraryFallbackSloBurn` when `oya_ontology_ontology_read_library_fallback_lag_seconds > 300` for 2 consecutive evaluator windows.
- Open a sev2 if `oya_ontology_ontology_read_library_fallback_correctness_ratio < 0.9999` and the affected label set includes `tenant_id`, `cell_id`, or `principal_id`.
- Open a sev1 if `oya_ontology_ontology_read_library_fallback_queue_depth > 5000` for 15 minutes or retry backlog grows by more than 20 percent in one 5 minute window.
- Trigger from customer report when Support tags the case `ontology.ontology-read-library-fallback.customer_visible` in Zendesk.
- Trigger from CI when `cargo run -p oya-dev-cli -- gate validate ontology-ontology-read-library-fallback --production-snapshot` exits non-zero against the latest production evidence bundle.
- Primary dashboard: `https://grafana.dev.oyatie.internal/d/ontology-substrate/ontology-read-library-fallback?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=116`.
- Secondary dashboard: `https://grafana.dev.oyatie.internal/d/ontology-substrate/ontology-read-library-fallback?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=213`.
- Loki explorer: `https://grafana.dev.oyatie.internal/explore?query={namespace="ontology",runbook="ontology-read-library-fallback"}`.
- Alertmanager route: `oyatie-ontology-ontology-read-library-fallback-critical`; silence only with incident commander approval and `EVT_ONTOLOGY_ONTOLOGY_READ_LIBRARY_FALLBACK_INCIDENT` evidence.
- Synthetic probe: `oya ops probe ontology ontology-read-library-fallback --cell prod-us-east-1 --tenant synthetic-canary` returns `healthy=true`.
- Drift detector: `registry/ontology/ontology-read-library-fallback/expected-state.json` hash differs from live `https://ontology.internal.oyatie.dev/v1/ontology/admin/state-hash`.
- Service-specific metric `oya_ontology_ontology_read_library_fallback_projection_lag_seconds` exceeds the threshold documented in `microservices/ontology/slos/function-read-latency.openslo.yaml`.

## Symptoms
- User-facing impact: object graphs, entity projections, share tokens, and cross-service semantic reads may be stale, slow, or cross-tenant unsafe.
- Operators see Grafana panel `type-registry-health.json / Ontology Read Library Fallback burn rate` turn red before the primary alert resolves.
- Loki signature `ontology.ontology_read_library_fallback.incident_state=failed` appears with fields `incident_id`, `tenant_id`, `cell_id`, `decision_id`, `evidence_hash`.
- Kubernetes events include `reason=OntologyOntologyReadLibraryFallbackDegraded` on deployment `ontology-ontology-read-library-fallback-worker` or `ontology-app`.
- Audit-chain shows missing or delayed `EVT_ONTOLOGY_ONTOLOGY_READ_LIBRARY_FALLBACK_INCIDENT` entries when queried with `oya audit-chain query --event-class EVT_ONTOLOGY_ONTOLOGY_READ_LIBRARY_FALLBACK_INCIDENT --since 30m`.
- Metric pattern: `oya_ontology_ontology_read_library_fallback_error_ratio` rises before `oya_ontology_ontology_read_library_fallback_lag_seconds`; if lag rises first, suspect dependency saturation rather than local regression.
- Metric pattern: `oya_ontology_ontology_read_library_fallback_queue_depth` increases while pod CPU stays below 40 percent; suspect downstream refusal, replay backlog, or feature flag deadlock.
- Tenant-specific shape: one `tenant_id` dominates labels in `oya_ontology_ontology_read_library_fallback_queue_depth`; isolate tenant before fleet mitigation.
- Fleet-wide shape: at least three cells report `OntologyOntologyReadLibraryFallbackCritical` in one 15 minute window; switch to cross-cell bridge even if individual tenants are low-volume.
- Log signature `decision=deny reason=ontology-read-library-fallback.policy_guard` means the guard is working; investigate caller inputs before rollback.
- Log signature `decision=permit reason=ontology-read-library-fallback.break_glass` means manual intervention is active; confirm two-person authorization.
- Log signature `audit_emit_status=stalled event_class=EVT_ONTOLOGY_ONTOLOGY_READ_LIBRARY_FALLBACK_INCIDENT` means mitigation cannot be closed until replay succeeds.
- Service-specific metric pattern: `oya_ontology_ontology_read_library_fallback_query_p99_seconds` rises while `oya_ontology_ontology_read_library_fallback_cross_tenant_refusal_total` is flat; inspect local worker health before escalating vendors.
- Service-specific metric pattern: `oya_ontology_ontology_read_library_fallback_cross_tenant_refusal_total` rises while `oya_ontology_ontology_read_library_fallback_error_ratio` is flat; suspect stale export, stale recommendation, stale projection, or vendor dependency lag.

## Diagnostic Steps
1. Set incident variables: `export INCIDENT_ID=INC-ontology-ontology-read-library-fallback-$(date -u +%Y%m%dT%H%M%SZ); export CELL=prod-us-east-1; export TENANT=synthetic-canary`.
2. Confirm active alerts: `curl -s https://ontology.internal.oyatie.dev/v1/ontology/alerts?runbook=ontology-read-library-fallback | jq .alerts`.
3. Check Kubernetes rollout: `kubectl -n ontology rollout status deploy/ontology-ontology-read-library-fallback-worker --timeout=60s`.
4. List unhealthy pods: `kubectl -n ontology get pods -l app=ontology-read-library-fallback -o wide`.
5. Read structured logs: `kubectl -n ontology logs deploy/ontology-ontology-read-library-fallback-worker --since=30m | rg "ontology.ontology_read_library_fallback.incident_state|OntologyOntologyReadLibraryFallbackCritical|EVT_ONTOLOGY_ONTOLOGY_READ_LIBRARY_FALLBACK_INCIDENT"`.
6. Query Loki directly: `logcli query '{namespace="ontology",runbook="ontology-read-library-fallback"}' --since=30m --limit=200`.
7. Check Prometheus error ratio: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_ontology_ontology_read_library_fallback_error_ratio{cell="prod-us-east-1"}'`.
8. Check lag: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_ontology_ontology_read_library_fallback_lag_seconds{cell="prod-us-east-1"}'`.
9. Check queue: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_ontology_ontology_read_library_fallback_queue_depth{cell="prod-us-east-1"}'`.
10. Check service-specific signal: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_ontology_ontology_read_library_fallback_projection_lag_seconds{cell="prod-us-east-1"}'`.
11. Open primary dashboard: `open "https://grafana.dev.oyatie.internal/d/ontology-substrate/ontology-read-library-fallback?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=116&var-incident=$INCIDENT_ID"`.
12. Open secondary dashboard: `open "https://grafana.dev.oyatie.internal/d/ontology-substrate/ontology-read-library-fallback?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=213&var-tenant=$TENANT"`.
13. Verify audit-chain emission: `oya audit-chain query --event-class EVT_ONTOLOGY_ONTOLOGY_READ_LIBRARY_FALLBACK_INCIDENT --since 30m --cell $CELL --tenant $TENANT`.
14. Verify service state: `oya ops ontology ontology-read-library-fallback status --cell $CELL --tenant $TENANT --output json`.
15. Run production snapshot gate: `cargo run -p oya-dev-cli -- gate validate ontology-ontology-read-library-fallback --production-snapshot --cell $CELL`.
16. Run crate smoke test: `cargo test -p oya-ontology-domain ontology_read_library_fallback -- --nocapture`.
17. Check API contract smoke: `curl -s https://ontology.internal.oyatie.dev/v1/ontology/ontology-read-library-fallback/incident-handoff -H "x-oya-tenant: $TENANT"`.
18. Inspect config: `test -f microservices/ontology/iac/kustomize/base/kustomization.yaml && sed -n '1,180p' microservices/ontology/iac/kustomize/base/kustomization.yaml`.
19. Inspect feature flags: `oya flags get oya.ontology.ontology_read_library_fallback.incident_hold --cell $CELL --tenant $TENANT --output yaml`.
20. Inspect circuit breaker: `oya ops breaker status ontology-ontology-read-library-fallback-circuit-breaker --cell $CELL --tenant $TENANT`.
21. Check recent deploy: `kubectl -n ontology rollout history deploy/ontology-ontology-read-library-fallback-worker | tail -20`.
22. Check policy file: `test -f microservices/ontology/policy/cross-tenant-refusal.cedar || find microservices/ontology/policy -maxdepth 2 -type f | sort`.
23. Check SLO files: `ls microservices/ontology/slos/*.openslo.yaml | sort | rg "function|ontology"`.
24. Check catalog components: `find microservices/ontology/catalog -maxdepth 1 -type f | sort | rg "object-type|entity-store|query-engine|share-token|read-path|cedar|action-engine"`.
25. Run targeted SQL state query: `psql $OYA_PROD_DSN -c "select incident_id, tenant_id, cell_id, state, updated_at from ontology_ontology_read_library_fallback_incidents where updated_at > now() - interval '30 minutes' order by updated_at desc limit 20;"`.
26. Confirm no cross-cell spread: `oya ops cells query --metric oya_ontology_ontology_read_library_fallback_error_ratio --window 30m --threshold 0.02`.
27. Snapshot evidence: `oya evidence snapshot --incident $INCIDENT_ID --microservice ontology --runbook ontology-read-library-fallback --output evidence/incidents/$INCIDENT_ID.json`.

### Diagnostic Decision Tree
```text
Ontology Read Library Fallback incident decision tree
1. Is OntologyOntologyReadLibraryFallbackCritical firing in more than one cell?
   |-- yes: declare fleet incident, page PagerDuty oya-ontology-primary; graph-platform secondary, and run cross-cell containment.
   |-- no: keep scope to the affected cell and continue tenant isolation checks.
2. Does oya_ontology_ontology_read_library_fallback_queue_depth grow while oya_ontology_ontology_read_library_fallback_error_ratio is flat?
   |-- yes: downstream dependency, replay backlog, or queue-drain issue; choose mitigation branch B.
   |-- no: local regression, bad input, or policy/config drift; continue branch selection.
3. Does audit-chain show EVT_ONTOLOGY_ONTOLOGY_READ_LIBRARY_FALLBACK_INCIDENT gaps?
   |-- yes: do not close; run evidence replay before resolution.
   |-- no: mitigation can proceed after state is green.
4. Is customer, finance, security, or regulator impact confirmed?
   |-- yes: promote severity, open #inc-ontology, and notify compliance or security handoff.
   |-- no: keep internal incident and collect evidence.
```
- Branch A (cross-tenant graph safety risk): use the matching mitigation block below and record `decision_branch=A` in `EVT_ONTOLOGY_ONTOLOGY_READ_LIBRARY_FALLBACK_INCIDENT`.
- Branch B (query or projection freshness degradation): use the matching mitigation block below and record `decision_branch=B` in `EVT_ONTOLOGY_ONTOLOGY_READ_LIBRARY_FALLBACK_INCIDENT`.
- Branch C (storage dependency degraded): use the matching mitigation block below and record `decision_branch=C` in `EVT_ONTOLOGY_ONTOLOGY_READ_LIBRARY_FALLBACK_INCIDENT`.
- Branch D (customer-visible semantic surface impact): use the matching mitigation block below and record `decision_branch=D` in `EVT_ONTOLOGY_ONTOLOGY_READ_LIBRARY_FALLBACK_INCIDENT`.

## Mitigation Steps
1. Acknowledge page: `pd incident ack --service ontology --incident $INCIDENT_ID`.
2. Create bridge: `oya incident bridge create --incident $INCIDENT_ID --channel #inc-ontology --severity sev2`.
3. Freeze risky automation: `oya flags set oya.ontology.ontology_read_library_fallback.incident_hold=true --cell $CELL --tenant $TENANT --reason $INCIDENT_ID`.
4. Enable circuit breaker: `oya ops breaker open ontology-ontology-read-library-fallback-circuit-breaker --cell $CELL --tenant $TENANT --ttl 30m --reason $INCIDENT_ID`.
5. Reduce blast radius: `kubectl -n ontology scale deploy/ontology-ontology-read-library-fallback-worker --replicas=1`.
6. Protect tenant boundary: `oya tenancy quarantine --tenant $TENANT --reason ontology-ontology-read-library-fallback --ttl 60m`.
7. Pause promotion: incident hold PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
8. Drain queue safely: `oya ops ontology ontology-read-library-fallback drain --cell $CELL --tenant $TENANT --max-items 500 --dry-run`.
9. Execute bounded drain: `oya ops ontology ontology-read-library-fallback drain --cell $CELL --tenant $TENANT --max-items 500 --confirm $INCIDENT_ID`.
10. Replay missing audit events: `oya audit-chain replay --event-class EVT_ONTOLOGY_ONTOLOGY_READ_LIBRARY_FALLBACK_INCIDENT --incident $INCIDENT_ID --from evidence/incidents/$INCIDENT_ID.json`.
11. Rollback last deploy if causal: `kubectl -n ontology rollout undo deploy/ontology-ontology-read-library-fallback-worker`.
12. Raise HPA cap if saturation is proven: `kubectl -n ontology patch hpa ontology-ontology-read-library-fallback-worker --type merge -p '{"spec":{"maxReplicas":12}}'`.
13. Throttle hot tenant: `oya ops rate-limit set --tenant $TENANT --surface ontology.ontology-read-library-fallback --rps 25 --ttl 30m`.
14. Block abusive principal when relevant: `oya identity principal suspend --principal suspected-abuse --tenant $TENANT --reason $INCIDENT_ID`.
15. Protect evidence: `oya evidence freeze --incident $INCIDENT_ID --paths microservices/ontology/runbooks/ontology-read-library-fallback.md,evidence/incidents/$INCIDENT_ID.json`.
16. Notify service owners: `oya notify service-owner --microservice ontology --incident $INCIDENT_ID --channel #inc-ontology`.
17. Open external vendor ticket: `oya vendor ticket open --vendor "ClickHouse support" --incident $INCIDENT_ID --summary ontology-ontology-read-library-fallback`.
18. Confirm breaker effect: `oya ops breaker status ontology-ontology-read-library-fallback-circuit-breaker --cell $CELL --tenant $TENANT --expect open`.
19. Confirm user impact reduced: `curl -s https://ontology.internal.oyatie.dev/v1/ontology/ontology-read-library-fallback/incident-handoff/health -H "x-oya-tenant: $TENANT"`.
20. Emit mitigation audit: `oya audit-chain emit --event-class EVT_ONTOLOGY_ONTOLOGY_READ_LIBRARY_FALLBACK_INCIDENT --incident $INCIDENT_ID --field mitigation=active --field runbook=ontology-read-library-fallback`.

### Mitigation Branch Guidance
- Branch A: cross-tenant graph safety risk.
  - Required action: keep `ontology-ontology-read-library-fallback-circuit-breaker` open until `oya_ontology_ontology_read_library_fallback_error_ratio` is below 0.005 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/ontology-substrate/ontology-read-library-fallback?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=117` to the incident.
  - Required audit: emit `EVT_ONTOLOGY_ONTOLOGY_READ_LIBRARY_FALLBACK_INCIDENT` with `branch=A`, `operator_id`, and `evidence_hash`.
- Branch B: query or projection freshness degradation.
  - Required action: keep `ontology-ontology-read-library-fallback-circuit-breaker` open until `oya_ontology_ontology_read_library_fallback_error_ratio` is below 0.01 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/ontology-substrate/ontology-read-library-fallback?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=118` to the incident.
  - Required audit: emit `EVT_ONTOLOGY_ONTOLOGY_READ_LIBRARY_FALLBACK_INCIDENT` with `branch=B`, `operator_id`, and `evidence_hash`.
- Branch C: storage dependency degraded.
  - Required action: keep `ontology-ontology-read-library-fallback-circuit-breaker` open until `oya_ontology_ontology_read_library_fallback_error_ratio` is below 0.005 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/ontology-substrate/ontology-read-library-fallback?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=119` to the incident.
  - Required audit: emit `EVT_ONTOLOGY_ONTOLOGY_READ_LIBRARY_FALLBACK_INCIDENT` with `branch=C`, `operator_id`, and `evidence_hash`.
- Branch D: customer-visible semantic surface impact.
  - Required action: keep `ontology-ontology-read-library-fallback-circuit-breaker` open until `oya_ontology_ontology_read_library_fallback_error_ratio` is below 0.01 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/ontology-substrate/ontology-read-library-fallback?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=120` to the incident.
  - Required audit: emit `EVT_ONTOLOGY_ONTOLOGY_READ_LIBRARY_FALLBACK_INCIDENT` with `branch=D`, `operator_id`, and `evidence_hash`.

## Resolution Steps
1. Identify code owner path: `rg "ontology_read_library_fallback|OntologyOntologyReadLibraryFallbackCritical|ontology.ontology_read_library_fallback.incident_state" crates microservices/ontology -g "!microservices/ontology/runbooks/**"`.
2. Patch domain invariant: `edit oya-ontology-domain where ontology_read_library_fallback state transition is validated`.
3. Patch API guard: `edit microservices/ontology/contracts/openapi/ontology.yaml if the failing path is north-south or async handoff`.
4. Patch policy: `edit microservices/ontology/policy/cross-tenant-refusal.cedar with explicit deny/permit branch and tenant/cell scope`.
5. Patch runtime config: `edit microservices/ontology/iac/kustomize/base/kustomization.yaml if deploy/config drift caused the incident`.
6. Add regression test: `cargo test -p oya-ontology-domain ontology_read_library_fallback_incident_regression -- --nocapture`.
7. Add gate evidence: `cargo run -p oya-dev-cli -- gate validate ontology-ontology-read-library-fallback --fixture incident-ontology-read-library-fallback.json`.
8. Add SLO assertion: `update microservices/ontology/slos/function-read-latency.openslo.yaml with alert OntologyOntologyReadLibraryFallbackCritical when this was a missing alert`.
9. Add dashboard panel: `update microservices/ontology/dashboards/type-registry-health.json with oya_ontology_ontology_read_library_fallback_error_ratio, oya_ontology_ontology_read_library_fallback_lag_seconds, and oya_ontology_ontology_read_library_fallback_queue_depth`.
10. Rebuild affected crate: `cargo check -p oya-ontology-domain --all-targets`.
11. Run targeted tests: `cargo test -p oya-ontology-domain --all-features`.
12. Run policy validation: `cargo run -p oya-dev-cli -- gate validate ontology-policy --microservice ontology`.
13. Deploy canary: `oya deploy canary --microservice ontology --component ontology-ontology-read-library-fallback-worker --cell $CELL --weight 1`.
14. Watch burn rate: `oya ops watch --metric oya_ontology_ontology_read_library_fallback_error_ratio --threshold 0.005 --window 30m --cell $CELL`.
15. Close circuit breaker: `oya ops breaker close ontology-ontology-read-library-fallback-circuit-breaker --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
16. Unfreeze automation: `oya flags set oya.ontology.ontology_read_library_fallback.incident_hold=false --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
17. Resume promotion: recovery PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
18. Seal resolution audit: `oya audit-chain emit --event-class EVT_ONTOLOGY_ONTOLOGY_READ_LIBRARY_FALLBACK_INCIDENT --incident $INCIDENT_ID --field resolution=complete --field runbook=ontology-read-library-fallback`.
19. Verify seal: `oya audit-chain verify --event-class EVT_ONTOLOGY_ONTOLOGY_READ_LIBRARY_FALLBACK_INCIDENT --incident $INCIDENT_ID`.
20. Attach final evidence: `oya evidence attach --incident $INCIDENT_ID --file evidence/incidents/$INCIDENT_ID.json --kind final-resolution`.

### Code Paths To Inspect First
- `oya-ontology-domain`: inspect for `ontology_read_library_fallback` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `oya-ontology-kernel`: inspect for `ontology_read_library_fallback` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `oya-ontology-api`: inspect for `ontology_read_library_fallback` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `oya-check-ontology-projection-coverage`: inspect for `ontology_read_library_fallback` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `microservices/ontology/contracts/openapi/ontology.yaml`: verify request/response or event contract only when incident evidence points there.
- `microservices/ontology/contracts/asyncapi/ontology-events.yaml`: verify request/response or event contract only when incident evidence points there.
- `microservices/ontology/contracts/proto/ontology.proto`: verify request/response or event contract only when incident evidence points there.
- `microservices/ontology/dashboards/type-registry-health.json`: verify panel coverage for `oya_ontology_ontology_read_library_fallback_error_ratio`, `oya_ontology_ontology_read_library_fallback_lag_seconds`, and `oya_ontology_ontology_read_library_fallback_projection_lag_seconds`.
- `microservices/ontology/slos/`: verify alert vocabulary and threshold alignment before changing runtime thresholds.
- `microservices/ontology/policy/`: verify policy branch ownership before relaxing deny rules or emergency bypasses.

## Verification Checklist
- `OntologyOntologyReadLibraryFallbackCritical` and `OntologyOntologyReadLibraryFallbackSloBurn` are both resolved in Alertmanager for 30 minutes.
- `oya_ontology_ontology_read_library_fallback_error_ratio < 0.005` for 3 consecutive 10 minute windows.
- `oya_ontology_ontology_read_library_fallback_lag_seconds < 120` for all production cells.
- `oya_ontology_ontology_read_library_fallback_queue_depth` is draining and not growing for the affected tenant.
- Service-specific signal `oya_ontology_ontology_read_library_fallback_projection_lag_seconds` is below the threshold documented in `microservices/ontology/slos/function-read-latency.openslo.yaml`.
- dashboard `https://grafana.dev.oyatie.internal/d/ontology-substrate/ontology-read-library-fallback?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=116` shows green panels for the affected cell.
- audit-chain query for `EVT_ONTOLOGY_ONTOLOGY_READ_LIBRARY_FALLBACK_INCIDENT` returns mitigation and resolution events.
- circuit breaker `ontology-ontology-read-library-fallback-circuit-breaker` is closed after rollback window.
- feature flag `oya.ontology.ontology_read_library_fallback.incident_hold` is false for the affected tenant unless long-term hold is approved.
- runbook invocation evidence is attached to `evidence/incidents/$INCIDENT_ID.json`.
- service owner acknowledged final handoff in `#inc-ontology`.

## Postmortem Template
Use this exact skeleton for the incident document. The field names are intentionally stable for ADR-0263 audit emission extraction.
```markdown
---
doc_class: IncidentPostmortem
runbook_id: ontology-ontology-read-library-fallback
microservice: ontology
event_class: EVT_ONTOLOGY_ONTOLOGY_READ_LIBRARY_FALLBACK_INCIDENT
incident_id: <INC-...>
severity: sev2
status: draft
detected_at: <UTC>
mitigated_at: <UTC>
resolved_at: <UTC>
commander: <handle>
evidence_hash: <sha256>
---

# Ontology Read Library Fallback postmortem

## Summary
- What happened in ontology/ontology-read-library-fallback.
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
- Emit EVT_ONTOLOGY_ONTOLOGY_READ_LIBRARY_FALLBACK_INCIDENT with incident_id, tenant_id, cell_id, principal_id, decision_id, evidence_hash, operator_id, runbook_id.
- Attach dashboard snapshot URLs and command transcripts.
- Seal mitigation and resolution events before closure.

## Corrective Actions
- Action, owner, due date, validation command, linked issue.
```

## Escalation Path
- Primary on-call: PagerDuty oya-ontology-primary; graph-platform secondary.
- Incident SLA: ack 3m for sev0/sev1, 10m for sev2, 30m for sev3; status update every 10m until `OntologyOntologyReadLibraryFallbackCritical` clears.
- Incident commander: first responder from axis-ontology + ops-sre-reliability + data-boundary-security; transfer only by explicit message in `#inc-ontology`.
- Security escalation: page `ops-security-primary` immediately for sev0, credential, cross-tenant, fraud, or audit-seal symptoms.
- Compliance escalation: page `dpo-office-duty` when tenant data, regulator evidence, money movement, or breach-clock symptoms are present.
- Architecture escalation: page `council-architecture-reviewer` before manual bypass, policy rollback, or invariant relaxation.
- External vendors: ClickHouse support; Oracle PostgreSQL support; Cedar policy runtime support. Open a ticket once local dependency health is proven and vendor dependency remains suspect.
- Customer communications: use status page component `oyatie-ontology-ontology-read-library-fallback` and keep private details in the incident channel.
- Regulatory clock: if tenant data, financial correctness, or evidence integrity is possibly affected, start the compliance 72h assessment timer even if exposure is unconfirmed.
- Executive notice: sev0 or fleet-wide sev1 goes to `#exec-incident-readout` within 30 minutes.

## Cross-µservice Coordination
- Notify `tenancy`: `oya incident handoff --target tenancy --source ontology --runbook ontology-read-library-fallback --incident $INCIDENT_ID --severity sev2 --branch A`; expect `202 accepted`.
- Require `tenancy` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `audit-chain`: `oya incident handoff --target audit-chain --source ontology --runbook ontology-read-library-fallback --incident $INCIDENT_ID --severity sev2 --branch B`; expect `202 accepted`.
- Require `audit-chain` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `intelligence`: `oya incident handoff --target intelligence --source ontology --runbook ontology-read-library-fallback --incident $INCIDENT_ID --severity sev2 --branch C`; expect `202 accepted`.
- Require `intelligence` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `governance`: `oya incident handoff --target governance --source ontology --runbook ontology-read-library-fallback --incident $INCIDENT_ID --severity sev2 --branch D`; expect `202 accepted`.
- Require `governance` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Observability handoff API: `oya incident handoff --target observability --source ontology --runbook ontology-read-library-fallback --incident $INCIDENT_ID`.
- Governance handoff API: `oya incident handoff --target governance --source ontology --runbook ontology-read-library-fallback --incident $INCIDENT_ID`.
- Compliance handoff API: `oya incident handoff --target compliance --source ontology --runbook ontology-read-library-fallback --incident $INCIDENT_ID`.
- Audit-chain handoff API: `oya incident handoff --target audit-chain --source ontology --runbook ontology-read-library-fallback --incident $INCIDENT_ID`.
- Tenancy handoff API: `oya incident handoff --target tenancy --source ontology --runbook ontology-read-library-fallback --incident $INCIDENT_ID`.

## Handoff Notes
- Do not hand off with only the alert name; include `oya_ontology_ontology_read_library_fallback_error_ratio`, `oya_ontology_ontology_read_library_fallback_lag_seconds`, `oya_ontology_ontology_read_library_fallback_queue_depth`, `oya_ontology_ontology_read_library_fallback_projection_lag_seconds`, current breaker state, and audit seal status.
- Keep `ontology-ontology-read-library-fallback-circuit-breaker` owner as axis-ontology + ops-sre-reliability + data-boundary-security until the receiving service explicitly accepts.
- If another runbook owns the downstream fix, link this incident as upstream and keep this runbook open until downstream verification returns green.
- Close only after `EVT_ONTOLOGY_ONTOLOGY_READ_LIBRARY_FALLBACK_INCIDENT` has a sealed resolution row and every coordination endpoint above has either accepted or explicitly declined scope.

## Sources Checked During This Substance Pass
- `microservices/ontology/dashboards/` for dashboard names and operational panels: type-registry-health.json, query-latency.json, read-path-library-freshness.json, cedar-policy-coverage.json.
- `microservices/ontology/slos/` for OpenSLO alert vocabulary and threshold alignment: function-read-latency.openslo.yaml, function-read-availability.openslo.yaml, dynamic-layer-freshness.openslo.yaml, audit-chain-emission-completeness.openslo.yaml.
- `microservices/ontology/policy/` for named policy and authorization surfaces: cross-tenant-refusal.cedar, ontology-write-quota.cedar, tenant-scope.cedar, type-isolation.md.
- `microservices/ontology/contracts/` for API, AsyncAPI, proto, and adapter surfaces: contracts/openapi/ontology.yaml, contracts/asyncapi/ontology-events.yaml, contracts/proto/ontology.proto.
- `microservices/ontology/catalog/` for component and owner vocabulary; existing runbook topic `ontology-read-library-fallback` was preserved as the scenario anchor.
