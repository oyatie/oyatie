---
doc_class: Runbook
title: RAG Retrieval Quality Regression
status: Accepted
date: 2026-05-20
microservice: intelligence
severity: sev2
audience: oncall-engineer
owner_team: axis-intelligence + ops-sre-reliability + ai-safety
source_wave: codex-runbooks-substrate-w2
change_scope: net-new operational scenario
doc_status: published
---

# Runbook: RAG Retrieval Quality Regression

## Operator Contract
- Runbook id: intelligence-rag-retrieval-quality-regression.
- Primary service namespace: `intelligence`.
- Owning rotation: PagerDuty oya-intelligence-primary; ai-safety secondary.
- Incident channel: `#inc-intelligence`.
- Operational focus: protecting model dispatch, prompt safety, provider credentials, RAG evidence, refusal correctness, and audit taps while resolving rag retrieval quality regression.
- External dependencies: OpenAI support; Anthropic support; Google Cloud Vertex AI support; OpenBao operations.
- API authority: `https://intelligence.internal.oyatie.dev/v1/intelligence/rag-retrieval-quality-regression/incident-handoff`.
- Audit event class: `EVT_INTELLIGENCE_RAG_RETRIEVAL_QUALITY_REGRESSION_INCIDENT` with ADR-0263 fields `incident_id`, `tenant_id`, `cell_id`, `microservice`, `runbook_id`, `decision_id`, `evidence_hash`, `operator_id`.
- Stop condition: mitigation has held for 30 minutes, `IntelligenceRAGRetrievalQualityRegressionCritical` is green, and every handoff API in Cross-microservice Coordination returns `202 accepted`.
- Safety invariant: never clear the incident until `EVT_INTELLIGENCE_RAG_RETRIEVAL_QUALITY_REGRESSION_INCIDENT` is sealed and the postmortem skeleton exists under `evidence/postmortems/intelligence-rag-retrieval-quality-regression-<incident-id>.md`.

## Trigger Conditions
- Page on alert `IntelligenceRAGRetrievalQualityRegressionCritical` when `oya_intelligence_rag_retrieval_quality_regression_error_ratio > 0.02` for 10 minutes in any production cell.
- Page on alert `IntelligenceRAGRetrievalQualityRegressionSloBurn` when `oya_intelligence_rag_retrieval_quality_regression_lag_seconds > 300` for 2 consecutive evaluator windows.
- Open a sev2 if `oya_intelligence_rag_retrieval_quality_regression_correctness_ratio < 0.9999` and the affected label set includes `tenant_id`, `cell_id`, or `principal_id`.
- Open a sev1 if `oya_intelligence_rag_retrieval_quality_regression_queue_depth > 5000` for 15 minutes or retry backlog grows by more than 20 percent in one 5 minute window.
- Trigger from customer report when Support tags the case `intelligence.rag-retrieval-quality-regression.customer_visible` in Zendesk.
- Trigger from CI when `cargo run -p oya-dev-cli -- gate validate intelligence-rag-retrieval-quality-regression --production-snapshot` exits non-zero against the latest production evidence bundle.
- Primary dashboard: `https://grafana.dev.oyatie.internal/d/intelligence-substrate/rag-retrieval-quality-regression?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=116`.
- Secondary dashboard: `https://grafana.dev.oyatie.internal/d/intelligence-substrate/rag-retrieval-quality-regression?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=213`.
- Loki explorer: `https://grafana.dev.oyatie.internal/explore?query={namespace="intelligence",runbook="rag-retrieval-quality-regression"}`.
- Alertmanager route: `oyatie-intelligence-rag-retrieval-quality-regression-critical`; silence only with incident commander approval and `EVT_INTELLIGENCE_RAG_RETRIEVAL_QUALITY_REGRESSION_INCIDENT` evidence.
- Synthetic probe: `oya ops probe intelligence rag-retrieval-quality-regression --cell prod-us-east-1 --tenant synthetic-canary` returns `healthy=true`.
- Drift detector: `registry/intelligence/rag-retrieval-quality-regression/expected-state.json` hash differs from live `https://intelligence.internal.oyatie.dev/v1/intelligence/admin/state-hash`.
- Service-specific metric `oya_intelligence_rag_retrieval_quality_regression_provider_timeout_ratio` exceeds the threshold documented in `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`.

## Symptoms
- User-facing impact: AI drafts, model dispatch, refusal decisions, RAG answers, and provider routing may time out, leak policy intent, or produce unsafe responses.
- Operators see Grafana panel `provider-latency-heatmap.json / RAG Retrieval Quality Regression burn rate` turn red before the primary alert resolves.
- Loki signature `intelligence.rag_retrieval_quality_regression.incident_state=failed` appears with fields `incident_id`, `tenant_id`, `cell_id`, `decision_id`, `evidence_hash`.
- Kubernetes events include `reason=IntelligenceRAGRetrievalQualityRegressionDegraded` on deployment `intelligence-rag-retrieval-quality-regression-worker` or `intelligence`.
- Audit-chain shows missing or delayed `EVT_INTELLIGENCE_RAG_RETRIEVAL_QUALITY_REGRESSION_INCIDENT` entries when queried with `oya audit-chain query --event-class EVT_INTELLIGENCE_RAG_RETRIEVAL_QUALITY_REGRESSION_INCIDENT --since 30m`.
- Metric pattern: `oya_intelligence_rag_retrieval_quality_regression_error_ratio` rises before `oya_intelligence_rag_retrieval_quality_regression_lag_seconds`; if lag rises first, suspect dependency saturation rather than local regression.
- Metric pattern: `oya_intelligence_rag_retrieval_quality_regression_queue_depth` increases while pod CPU stays below 40 percent; suspect downstream refusal, replay backlog, or feature flag deadlock.
- Tenant-specific shape: one `tenant_id` dominates labels in `oya_intelligence_rag_retrieval_quality_regression_queue_depth`; isolate tenant before fleet mitigation.
- Fleet-wide shape: at least three cells report `IntelligenceRAGRetrievalQualityRegressionCritical` in one 15 minute window; switch to cross-cell bridge even if individual tenants are low-volume.
- Log signature `decision=deny reason=rag-retrieval-quality-regression.policy_guard` means the guard is working; investigate caller inputs before rollback.
- Log signature `decision=permit reason=rag-retrieval-quality-regression.break_glass` means manual intervention is active; confirm two-person authorization.
- Log signature `audit_emit_status=stalled event_class=EVT_INTELLIGENCE_RAG_RETRIEVAL_QUALITY_REGRESSION_INCIDENT` means mitigation cannot be closed until replay succeeds.
- Service-specific metric pattern: `oya_intelligence_rag_retrieval_quality_regression_refusal_error_ratio` rises while `oya_intelligence_rag_retrieval_quality_regression_rag_hit_quality_ratio` is flat; inspect local worker health before escalating vendors.
- Service-specific metric pattern: `oya_intelligence_rag_retrieval_quality_regression_rag_hit_quality_ratio` rises while `oya_intelligence_rag_retrieval_quality_regression_error_ratio` is flat; suspect stale export, stale recommendation, stale projection, or vendor dependency lag.

## Diagnostic Steps
1. Set incident variables: `export INCIDENT_ID=INC-intelligence-rag-retrieval-quality-regression-$(date -u +%Y%m%dT%H%M%SZ); export CELL=prod-us-east-1; export TENANT=synthetic-canary`.
2. Confirm active alerts: `curl -s https://intelligence.internal.oyatie.dev/v1/intelligence/alerts?runbook=rag-retrieval-quality-regression | jq .alerts`.
3. Check Kubernetes rollout: `kubectl -n intelligence rollout status deploy/intelligence-rag-retrieval-quality-regression-worker --timeout=60s`.
4. List unhealthy pods: `kubectl -n intelligence get pods -l app=rag-retrieval-quality-regression -o wide`.
5. Read structured logs: `kubectl -n intelligence logs deploy/intelligence-rag-retrieval-quality-regression-worker --since=30m | rg "intelligence.rag_retrieval_quality_regression.incident_state|IntelligenceRAGRetrievalQualityRegressionCritical|EVT_INTELLIGENCE_RAG_RETRIEVAL_QUALITY_REGRESSION_INCIDENT"`.
6. Query Loki directly: `logcli query '{namespace="intelligence",runbook="rag-retrieval-quality-regression"}' --since=30m --limit=200`.
7. Check Prometheus error ratio: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_intelligence_rag_retrieval_quality_regression_error_ratio{cell="prod-us-east-1"}'`.
8. Check lag: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_intelligence_rag_retrieval_quality_regression_lag_seconds{cell="prod-us-east-1"}'`.
9. Check queue: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_intelligence_rag_retrieval_quality_regression_queue_depth{cell="prod-us-east-1"}'`.
10. Check service-specific signal: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_intelligence_rag_retrieval_quality_regression_provider_timeout_ratio{cell="prod-us-east-1"}'`.
11. Open primary dashboard: `open "https://grafana.dev.oyatie.internal/d/intelligence-substrate/rag-retrieval-quality-regression?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=116&var-incident=$INCIDENT_ID"`.
12. Open secondary dashboard: `open "https://grafana.dev.oyatie.internal/d/intelligence-substrate/rag-retrieval-quality-regression?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=213&var-tenant=$TENANT"`.
13. Verify audit-chain emission: `oya audit-chain query --event-class EVT_INTELLIGENCE_RAG_RETRIEVAL_QUALITY_REGRESSION_INCIDENT --since 30m --cell $CELL --tenant $TENANT`.
14. Verify service state: `oya ops intelligence rag-retrieval-quality-regression status --cell $CELL --tenant $TENANT --output json`.
15. Run production snapshot gate: `cargo run -p oya-dev-cli -- gate validate intelligence-rag-retrieval-quality-regression --production-snapshot --cell $CELL`.
16. Run crate smoke test: `cargo test -p oya-intelligence-adapter-openai-api-kernel rag_retrieval_quality_regression -- --nocapture`.
17. Check API contract smoke: `curl -s https://intelligence.internal.oyatie.dev/v1/intelligence/rag-retrieval-quality-regression/incident-handoff -H "x-oya-tenant: $TENANT"`.
18. Inspect config: `test -f microservices/intelligence/iac/k8s/deployment.yaml && sed -n '1,180p' microservices/intelligence/iac/k8s/deployment.yaml`.
19. Inspect feature flags: `oya flags get oya.intelligence.rag_retrieval_quality_regression.incident_hold --cell $CELL --tenant $TENANT --output yaml`.
20. Inspect circuit breaker: `oya ops breaker status intelligence-rag-retrieval-quality-regression-circuit-breaker --cell $CELL --tenant $TENANT`.
21. Check recent deploy: `kubectl -n intelligence rollout history deploy/intelligence-rag-retrieval-quality-regression-worker | tail -20`.
22. Check policy file: `test -f microservices/intelligence/policy/provider-routing.cedar || find microservices/intelligence/policy -maxdepth 2 -type f | sort`.
23. Check SLO files: `ls microservices/intelligence/slos/*.openslo.yaml | sort | rg "dispatch|rag"`.
24. Check catalog components: `find microservices/intelligence/catalog -maxdepth 1 -type f | sort | rg "provider|model-routing|guardrails|credential|audit|eval|attribution"`.
25. Run targeted SQL state query: `psql $OYA_PROD_DSN -c "select incident_id, tenant_id, cell_id, state, updated_at from intelligence_rag_retrieval_quality_regression_incidents where updated_at > now() - interval '30 minutes' order by updated_at desc limit 20;"`.
26. Confirm no cross-cell spread: `oya ops cells query --metric oya_intelligence_rag_retrieval_quality_regression_error_ratio --window 30m --threshold 0.02`.
27. Snapshot evidence: `oya evidence snapshot --incident $INCIDENT_ID --microservice intelligence --runbook rag-retrieval-quality-regression --output evidence/incidents/$INCIDENT_ID.json`.

### Diagnostic Decision Tree
```text
RAG Retrieval Quality Regression incident decision tree
1. Is IntelligenceRAGRetrievalQualityRegressionCritical firing in more than one cell?
   |-- yes: declare fleet incident, page PagerDuty oya-intelligence-primary; ai-safety secondary, and run cross-cell containment.
   |-- no: keep scope to the affected cell and continue tenant isolation checks.
2. Does oya_intelligence_rag_retrieval_quality_regression_queue_depth grow while oya_intelligence_rag_retrieval_quality_regression_error_ratio is flat?
   |-- yes: downstream dependency, replay backlog, or queue-drain issue; choose mitigation branch B.
   |-- no: local regression, bad input, or policy/config drift; continue branch selection.
3. Does audit-chain show EVT_INTELLIGENCE_RAG_RETRIEVAL_QUALITY_REGRESSION_INCIDENT gaps?
   |-- yes: do not close; run evidence replay before resolution.
   |-- no: mitigation can proceed after state is green.
4. Is customer, finance, security, or regulator impact confirmed?
   |-- yes: promote severity, open #inc-intelligence, and notify compliance or security handoff.
   |-- no: keep internal incident and collect evidence.
```
- Branch A (safety or prompt boundary risk): use the matching mitigation block below and record `decision_branch=A` in `EVT_INTELLIGENCE_RAG_RETRIEVAL_QUALITY_REGRESSION_INCIDENT`.
- Branch B (provider dependency degraded): use the matching mitigation block below and record `decision_branch=B` in `EVT_INTELLIGENCE_RAG_RETRIEVAL_QUALITY_REGRESSION_INCIDENT`.
- Branch C (provider-credential BYOK or credential resolution risk, ADR-0255 §D-4): use the matching mitigation block below and record `decision_branch=C` in `EVT_INTELLIGENCE_RAG_RETRIEVAL_QUALITY_REGRESSION_INCIDENT`.
- Branch D (customer-visible AI workflow impact): use the matching mitigation block below and record `decision_branch=D` in `EVT_INTELLIGENCE_RAG_RETRIEVAL_QUALITY_REGRESSION_INCIDENT`.

## Mitigation
1. Acknowledge page: `pd incident ack --service intelligence --incident $INCIDENT_ID`.
2. Create bridge: `oya incident bridge create --incident $INCIDENT_ID --channel #inc-intelligence --severity sev2`.
3. Freeze risky automation: `oya flags set oya.intelligence.rag_retrieval_quality_regression.incident_hold=true --cell $CELL --tenant $TENANT --reason $INCIDENT_ID`.
4. Enable circuit breaker: `oya ops breaker open intelligence-rag-retrieval-quality-regression-circuit-breaker --cell $CELL --tenant $TENANT --ttl 30m --reason $INCIDENT_ID`.
5. Reduce blast radius: `kubectl -n intelligence scale deploy/intelligence-rag-retrieval-quality-regression-worker --replicas=1`.
6. Protect tenant boundary: `oya tenancy quarantine --tenant $TENANT --reason intelligence-rag-retrieval-quality-regression --ttl 60m`.
7. Pause promotion: incident hold PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
8. Drain queue safely: `oya ops intelligence rag-retrieval-quality-regression drain --cell $CELL --tenant $TENANT --max-items 500 --dry-run`.
9. Execute bounded drain: `oya ops intelligence rag-retrieval-quality-regression drain --cell $CELL --tenant $TENANT --max-items 500 --confirm $INCIDENT_ID`.
10. Replay missing audit events: `oya audit-chain replay --event-class EVT_INTELLIGENCE_RAG_RETRIEVAL_QUALITY_REGRESSION_INCIDENT --incident $INCIDENT_ID --from evidence/incidents/$INCIDENT_ID.json`.
11. Rollback last deploy if causal: `kubectl -n intelligence rollout undo deploy/intelligence-rag-retrieval-quality-regression-worker`.
12. Raise HPA cap if saturation is proven: `kubectl -n intelligence patch hpa intelligence-rag-retrieval-quality-regression-worker --type merge -p '{"spec":{"maxReplicas":12}}'`.
13. Throttle hot tenant: `oya ops rate-limit set --tenant $TENANT --surface intelligence.rag-retrieval-quality-regression --rps 25 --ttl 30m`.
14. Block abusive principal when relevant: `oya identity principal suspend --principal suspected-abuse --tenant $TENANT --reason $INCIDENT_ID`.
15. Protect evidence: `oya evidence freeze --incident $INCIDENT_ID --paths microservices/intelligence/runbooks/rag-retrieval-quality-regression.md,evidence/incidents/$INCIDENT_ID.json`.
16. Notify service owners: `oya notify service-owner --microservice intelligence --incident $INCIDENT_ID --channel #inc-intelligence`.
17. Open external vendor ticket: `oya vendor ticket open --vendor "OpenAI support" --incident $INCIDENT_ID --summary intelligence-rag-retrieval-quality-regression`.
18. Confirm breaker effect: `oya ops breaker status intelligence-rag-retrieval-quality-regression-circuit-breaker --cell $CELL --tenant $TENANT --expect open`.
19. Confirm user impact reduced: `curl -s https://intelligence.internal.oyatie.dev/v1/intelligence/rag-retrieval-quality-regression/incident-handoff/health -H "x-oya-tenant: $TENANT"`.
20. Emit mitigation audit: `oya audit-chain emit --event-class EVT_INTELLIGENCE_RAG_RETRIEVAL_QUALITY_REGRESSION_INCIDENT --incident $INCIDENT_ID --field mitigation=active --field runbook=rag-retrieval-quality-regression`.

### Mitigation Branch Guidance
- Branch A: safety or prompt boundary risk.
  - Required action: keep `intelligence-rag-retrieval-quality-regression-circuit-breaker` open until `oya_intelligence_rag_retrieval_quality_regression_error_ratio` is below 0.005 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/intelligence-substrate/rag-retrieval-quality-regression?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=117` to the incident.
  - Required audit: emit `EVT_INTELLIGENCE_RAG_RETRIEVAL_QUALITY_REGRESSION_INCIDENT` with `branch=A`, `operator_id`, and `evidence_hash`.
- Branch B: provider dependency degraded.
  - Required action: keep `intelligence-rag-retrieval-quality-regression-circuit-breaker` open until `oya_intelligence_rag_retrieval_quality_regression_error_ratio` is below 0.01 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/intelligence-substrate/rag-retrieval-quality-regression?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=118` to the incident.
  - Required audit: emit `EVT_INTELLIGENCE_RAG_RETRIEVAL_QUALITY_REGRESSION_INCIDENT` with `branch=B`, `operator_id`, and `evidence_hash`.
- Branch C: provider-credential BYOK or credential resolution risk (ADR-0255 §D-4).
  - Required action: keep `intelligence-rag-retrieval-quality-regression-circuit-breaker` open until `oya_intelligence_rag_retrieval_quality_regression_error_ratio` is below 0.005 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/intelligence-substrate/rag-retrieval-quality-regression?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=119` to the incident.
  - Required audit: emit `EVT_INTELLIGENCE_RAG_RETRIEVAL_QUALITY_REGRESSION_INCIDENT` with `branch=C`, `operator_id`, and `evidence_hash`.
- Branch D: customer-visible AI workflow impact.
  - Required action: keep `intelligence-rag-retrieval-quality-regression-circuit-breaker` open until `oya_intelligence_rag_retrieval_quality_regression_error_ratio` is below 0.01 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/intelligence-substrate/rag-retrieval-quality-regression?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=120` to the incident.
  - Required audit: emit `EVT_INTELLIGENCE_RAG_RETRIEVAL_QUALITY_REGRESSION_INCIDENT` with `branch=D`, `operator_id`, and `evidence_hash`.

## Resolution
1. Identify code owner path: `rg "rag_retrieval_quality_regression|IntelligenceRAGRetrievalQualityRegressionCritical|intelligence.rag_retrieval_quality_regression.incident_state" crates microservices/intelligence -g "!microservices/intelligence/runbooks/**"`.
2. Patch domain invariant: `edit oya-intelligence-adapter-openai-api-kernel where rag_retrieval_quality_regression state transition is validated`.
3. Patch API guard: `edit microservices/intelligence/contracts/openapi/intelligence.yaml if the failing path is north-south or async handoff`.
4. Patch policy: `edit microservices/intelligence/policy/provider-routing.cedar with explicit deny/permit branch and tenant/cell scope`.
5. Patch runtime config: `edit microservices/intelligence/iac/k8s/deployment.yaml if deploy/config drift caused the incident`.
6. Add regression test: `cargo test -p oya-intelligence-adapter-openai-api-kernel rag_retrieval_quality_regression_incident_regression -- --nocapture`.
7. Add gate evidence: `cargo run -p oya-dev-cli -- gate validate intelligence-rag-retrieval-quality-regression --fixture incident-rag-retrieval-quality-regression.json`.
8. Add SLO assertion: `update microservices/intelligence/slos/dispatch-api-availability.openslo.yaml with alert IntelligenceRAGRetrievalQualityRegressionCritical when this was a missing alert`.
9. Add dashboard panel: `update microservices/intelligence/dashboards/provider-latency-heatmap.json with oya_intelligence_rag_retrieval_quality_regression_error_ratio, oya_intelligence_rag_retrieval_quality_regression_lag_seconds, and oya_intelligence_rag_retrieval_quality_regression_queue_depth`.
10. Rebuild affected crate: `cargo check -p oya-intelligence-adapter-openai-api-kernel --all-targets`.
11. Run targeted tests: `cargo test -p oya-intelligence-adapter-openai-api-kernel --all-features`.
12. Run policy validation: `cargo run -p oya-dev-cli -- gate validate intelligence-policy --microservice intelligence`.
13. Deploy canary: `oya deploy canary --microservice intelligence --component intelligence-rag-retrieval-quality-regression-worker --cell $CELL --weight 1`.
14. Watch burn rate: `oya ops watch --metric oya_intelligence_rag_retrieval_quality_regression_error_ratio --threshold 0.005 --window 30m --cell $CELL`.
15. Close circuit breaker: `oya ops breaker close intelligence-rag-retrieval-quality-regression-circuit-breaker --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
16. Unfreeze automation: `oya flags set oya.intelligence.rag_retrieval_quality_regression.incident_hold=false --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
17. Resume promotion: recovery PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
18. Seal resolution audit: `oya audit-chain emit --event-class EVT_INTELLIGENCE_RAG_RETRIEVAL_QUALITY_REGRESSION_INCIDENT --incident $INCIDENT_ID --field resolution=complete --field runbook=rag-retrieval-quality-regression`.
19. Verify seal: `oya audit-chain verify --event-class EVT_INTELLIGENCE_RAG_RETRIEVAL_QUALITY_REGRESSION_INCIDENT --incident $INCIDENT_ID`.
20. Attach final evidence: `oya evidence attach --incident $INCIDENT_ID --file evidence/incidents/$INCIDENT_ID.json --kind final-resolution`.

### Code Paths To Inspect First
- `oya-intelligence-adapter-openai-api-kernel`: inspect for `rag_retrieval_quality_regression` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `oya-intelligence-rag-api`: inspect for `rag_retrieval_quality_regression` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `oya-intelligence-rag-endpoint-kernel`: inspect for `rag_retrieval_quality_regression` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `oya-governance-eval-domain`: inspect for `rag_retrieval_quality_regression` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `microservices/intelligence/contracts/openapi/intelligence.yaml`: verify request/response or event contract only when incident evidence points there.
- `microservices/intelligence/contracts/asyncapi/intelligence-events.yaml`: verify request/response or event contract only when incident evidence points there.
- `microservices/intelligence/contracts/proto/intelligence.proto`: verify request/response or event contract only when incident evidence points there.
- `microservices/intelligence/contracts/provider-adapter-trait.md`: verify request/response or event contract only when incident evidence points there.
- `microservices/intelligence/dashboards/provider-latency-heatmap.json`: verify panel coverage for `oya_intelligence_rag_retrieval_quality_regression_error_ratio`, `oya_intelligence_rag_retrieval_quality_regression_lag_seconds`, and `oya_intelligence_rag_retrieval_quality_regression_provider_timeout_ratio`.
- `microservices/intelligence/slos/`: verify alert vocabulary and threshold alignment before changing runtime thresholds.
- `microservices/intelligence/policy/`: verify policy branch ownership before relaxing deny rules or emergency bypasses.

## Verification Checklist
- `IntelligenceRAGRetrievalQualityRegressionCritical` and `IntelligenceRAGRetrievalQualityRegressionSloBurn` are both resolved in Alertmanager for 30 minutes.
- `oya_intelligence_rag_retrieval_quality_regression_error_ratio < 0.005` for 3 consecutive 10 minute windows.
- `oya_intelligence_rag_retrieval_quality_regression_lag_seconds < 120` for all production cells.
- `oya_intelligence_rag_retrieval_quality_regression_queue_depth` is draining and not growing for the affected tenant.
- Service-specific signal `oya_intelligence_rag_retrieval_quality_regression_provider_timeout_ratio` is below the threshold documented in `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`.
- dashboard `https://grafana.dev.oyatie.internal/d/intelligence-substrate/rag-retrieval-quality-regression?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=116` shows green panels for the affected cell.
- audit-chain query for `EVT_INTELLIGENCE_RAG_RETRIEVAL_QUALITY_REGRESSION_INCIDENT` returns mitigation and resolution events.
- circuit breaker `intelligence-rag-retrieval-quality-regression-circuit-breaker` is closed after rollback window.
- feature flag `oya.intelligence.rag_retrieval_quality_regression.incident_hold` is false for the affected tenant unless long-term hold is approved.
- runbook invocation evidence is attached to `evidence/incidents/$INCIDENT_ID.json`.
- service owner acknowledged final handoff in `#inc-intelligence`.

## Postmortem Template
Use this exact skeleton for the incident document. The field names are intentionally stable for ADR-0263 audit emission extraction.
```markdown
---
doc_class: IncidentPostmortem
runbook_id: intelligence-rag-retrieval-quality-regression
microservice: intelligence
event_class: EVT_INTELLIGENCE_RAG_RETRIEVAL_QUALITY_REGRESSION_INCIDENT
incident_id: <INC-...>
severity: sev2
status: draft
detected_at: <UTC>
mitigated_at: <UTC>
resolved_at: <UTC>
commander: <handle>
evidence_hash: <sha256>
---

# RAG Retrieval Quality Regression postmortem

## Summary
- What happened in intelligence/rag-retrieval-quality-regression.
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
- Emit EVT_INTELLIGENCE_RAG_RETRIEVAL_QUALITY_REGRESSION_INCIDENT with incident_id, tenant_id, cell_id, principal_id, decision_id, evidence_hash, operator_id, runbook_id.
- Attach dashboard snapshot URLs and command transcripts.
- Seal mitigation and resolution events before closure.

## Corrective Actions
- Action, owner, due date, validation command, linked issue.
```

## Escalation Path
- Primary on-call: PagerDuty oya-intelligence-primary; ai-safety secondary.
- Incident SLA: ack 3m for sev0/sev1, 10m for sev2, 30m for sev3; status update every 10m until `IntelligenceRAGRetrievalQualityRegressionCritical` clears.
- Incident commander: first responder from axis-intelligence + ops-sre-reliability + ai-safety; transfer only by explicit message in `#inc-intelligence`.
- Security escalation: page `ops-security-primary` immediately for sev0, credential, cross-tenant, fraud, or audit-seal symptoms.
- Compliance escalation: page `dpo-office-duty` when tenant data, regulator evidence, money movement, or breach-clock symptoms are present.
- Architecture escalation: page `council-architecture-reviewer` before manual bypass, policy rollback, or invariant relaxation.
- External vendors: OpenAI support; Anthropic support; Google Cloud Vertex AI support; OpenBao operations. Open a ticket once local dependency health is proven and vendor dependency remains suspect.
- Customer communications: use status page component `oyatie-intelligence-rag-retrieval-quality-regression` and keep private details in the incident channel.
- Regulatory clock: if tenant data, financial correctness, or evidence integrity is possibly affected, start the compliance 72h assessment timer even if exposure is unconfirmed.
- Executive notice: sev0 or fleet-wide sev1 goes to `#exec-incident-readout` within 30 minutes.

## Cross-µservice Coordination
- Notify `identity`: `oya incident handoff --target identity --source intelligence --runbook rag-retrieval-quality-regression --incident $INCIDENT_ID --severity sev2 --branch A`; expect `202 accepted`.
- Require `identity` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `audit-chain`: `oya incident handoff --target audit-chain --source intelligence --runbook rag-retrieval-quality-regression --incident $INCIDENT_ID --severity sev2 --branch B`; expect `202 accepted`.
- Require `audit-chain` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `ontology`: `oya incident handoff --target ontology --source intelligence --runbook rag-retrieval-quality-regression --incident $INCIDENT_ID --severity sev2 --branch C`; expect `202 accepted`.
- Require `ontology` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `governance`: `oya incident handoff --target governance --source intelligence --runbook rag-retrieval-quality-regression --incident $INCIDENT_ID --severity sev2 --branch D`; expect `202 accepted`.
- Require `governance` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Observability handoff API: `oya incident handoff --target observability --source intelligence --runbook rag-retrieval-quality-regression --incident $INCIDENT_ID`.
- Governance handoff API: `oya incident handoff --target governance --source intelligence --runbook rag-retrieval-quality-regression --incident $INCIDENT_ID`.
- Compliance handoff API: `oya incident handoff --target compliance --source intelligence --runbook rag-retrieval-quality-regression --incident $INCIDENT_ID`.
- Audit-chain handoff API: `oya incident handoff --target audit-chain --source intelligence --runbook rag-retrieval-quality-regression --incident $INCIDENT_ID`.
- Tenancy handoff API: `oya incident handoff --target tenancy --source intelligence --runbook rag-retrieval-quality-regression --incident $INCIDENT_ID`.

## Handoff Notes
- Do not hand off with only the alert name; include `oya_intelligence_rag_retrieval_quality_regression_error_ratio`, `oya_intelligence_rag_retrieval_quality_regression_lag_seconds`, `oya_intelligence_rag_retrieval_quality_regression_queue_depth`, `oya_intelligence_rag_retrieval_quality_regression_provider_timeout_ratio`, current breaker state, and audit seal status.
- Keep `intelligence-rag-retrieval-quality-regression-circuit-breaker` owner as axis-intelligence + ops-sre-reliability + ai-safety until the receiving service explicitly accepts.
- If another runbook owns the downstream fix, link this incident as upstream and keep this runbook open until downstream verification returns green.
- Close only after `EVT_INTELLIGENCE_RAG_RETRIEVAL_QUALITY_REGRESSION_INCIDENT` has a sealed resolution row and every coordination endpoint above has either accepted or explicitly declined scope.

## Sources Checked During This Substance Pass
- `microservices/intelligence/dashboards/` for dashboard names and operational panels: provider-latency-heatmap.json, prompt-injection-detection.md, refusal-rate-by-pack.json, intelligence-overview.json.
- `microservices/intelligence/slos/` for OpenSLO alert vocabulary and threshold alignment: dispatch-api-availability.openslo.yaml, dispatch-api-latency.openslo.yaml, policy-refusal-correctness.openslo.yaml, first-token-latency.openslo.yaml.
- `microservices/intelligence/policy/` for named policy and authorization surfaces: provider-routing.cedar, refusal-baseline.cedar, byok-gating.cedar, eu-ai-act-high-risk.cedar, dispatch-authorization.cedar.
- `microservices/intelligence/contracts/` for API, AsyncAPI, proto, and adapter surfaces: contracts/openapi/intelligence.yaml, contracts/asyncapi/intelligence-events.yaml, contracts/proto/intelligence.proto, contracts/provider-adapter-trait.md.
- `microservices/intelligence/catalog/` for component and owner vocabulary; existing runbook topic `rag-retrieval-quality-regression` was preserved as the scenario anchor.
