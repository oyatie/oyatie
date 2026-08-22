---
doc_class: Runbook
title: KYC AML Screening Pipeline Stall
status: Accepted
date: 2026-05-20
microservice: payments
severity: sev0
audience: security-engineer
owner_team: axis-payments + ops-sre-reliability + ops-security
source_wave: codex-runbooks-substrate-w2
change_scope: net-new operational scenario
doc_status: published
---

# Runbook: KYC AML Screening Pipeline Stall

## Operator Contract
- Runbook id: payments-kyc-aml-screening-pipeline-stall.
- Primary service namespace: `payments`.
- Owning rotation: PagerDuty payments-primary; fraud-risk secondary.
- Incident channel: `#inc-payments`.
- Operational focus: protecting money movement, PSP state, fraud review, KYC/AML screening, and ledger reconciliation while resolving kyc aml screening pipeline stall.
- External dependencies: Stripe enterprise support; Adyen technical support; Visa DPS risk support; Korean FSS liaison desk.
- API authority: `https://payments.internal.oyatie.dev/v1/payments/kyc-aml-screening-pipeline-stall/incident-handoff`.
- Audit event class: `EVT_PAYMENTS_KYC_AML_SCREENING_PIPELINE_STALL_INCIDENT` with ADR-0263 fields `incident_id`, `tenant_id`, `cell_id`, `microservice`, `runbook_id`, `decision_id`, `evidence_hash`, `operator_id`.
- Stop condition: mitigation has held for 30 minutes, `PaymentsKYCAMLScreeningPipelineStallCritical` is green, and every handoff API in Cross-microservice Coordination returns `202 accepted`.
- Safety invariant: never clear the incident until `EVT_PAYMENTS_KYC_AML_SCREENING_PIPELINE_STALL_INCIDENT` is sealed and the postmortem skeleton exists under `evidence/postmortems/payments-kyc-aml-screening-pipeline-stall-<incident-id>.md`.

## Trigger Conditions
- Page on alert `PaymentsKYCAMLScreeningPipelineStallCritical` when `payments_kyc_aml_screening_pipeline_stall_error_ratio > 0.02` for 10 minutes in any production cell.
- Page on alert `PaymentsKYCAMLScreeningPipelineStallSloBurn` when `payments_kyc_aml_screening_pipeline_stall_lag_seconds > 300` for 2 consecutive evaluator windows.
- Open a sev0 if `payments_kyc_aml_screening_pipeline_stall_correctness_ratio < 0.9999` and the affected label set includes `tenant_id`, `cell_id`, or `principal_id`.
- Open a sev1 if `payments_kyc_aml_screening_pipeline_stall_queue_depth > 5000` for 15 minutes or retry backlog grows by more than 20 percent in one 5 minute window.
- Trigger from customer report when Support tags the case `payments.kyc-aml-screening-pipeline-stall.customer_visible` in Zendesk.
- Trigger from CI when `cargo run -p dev-cli -- gate validate payments-kyc-aml-screening-pipeline-stall --production-snapshot` exits non-zero against the latest production evidence bundle.
- Primary dashboard: `https://grafana.dev.oyatie.internal/d/payments-substrate/kyc-aml-screening-pipeline-stall?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=116`.
- Secondary dashboard: `https://grafana.dev.oyatie.internal/d/payments-substrate/kyc-aml-screening-pipeline-stall?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=213`.
- Loki explorer: `https://grafana.dev.oyatie.internal/explore?query={namespace="payments",runbook="kyc-aml-screening-pipeline-stall"}`.
- Alertmanager route: `oyatie-payments-kyc-aml-screening-pipeline-stall-critical`; silence only with incident commander approval and `EVT_PAYMENTS_KYC_AML_SCREENING_PIPELINE_STALL_INCIDENT` evidence.
- Synthetic probe: `oya ops probe payments kyc-aml-screening-pipeline-stall --cell prod-us-east-1 --tenant synthetic-canary` returns `healthy=true`.
- Drift detector: `registry/payments/kyc-aml-screening-pipeline-stall/expected-state.json` hash differs from live `https://payments.internal.oyatie.dev/v1/payments/admin/state-hash`.
- Service-specific metric `payments_kyc_aml_screening_pipeline_stall_psp_decline_ratio` exceeds the threshold documented in `microservices/payments/slos/charge-api-availability.openslo.yaml`.

## Symptoms
- User-facing impact: charges, refunds, payouts, subscription renewals, and dispute evidence may be delayed, duplicated, or blocked.
- Operators see Grafana panel `psp-routing.json / KYC AML Screening Pipeline Stall burn rate` turn red before the primary alert resolves.
- Loki signature `payments.kyc_aml_screening_pipeline_stall.incident_state=failed` appears with fields `incident_id`, `tenant_id`, `cell_id`, `decision_id`, `evidence_hash`.
- Kubernetes events include `reason=PaymentsKYCAMLScreeningPipelineStallDegraded` on deployment `payments-kyc-aml-screening-pipeline-stall-worker` or `payments-app`.
- Audit-chain shows missing or delayed `EVT_PAYMENTS_KYC_AML_SCREENING_PIPELINE_STALL_INCIDENT` entries when queried with `oya audit-chain query --event-class EVT_PAYMENTS_KYC_AML_SCREENING_PIPELINE_STALL_INCIDENT --since 30m`.
- Metric pattern: `payments_kyc_aml_screening_pipeline_stall_error_ratio` rises before `payments_kyc_aml_screening_pipeline_stall_lag_seconds`; if lag rises first, suspect dependency saturation rather than local regression.
- Metric pattern: `payments_kyc_aml_screening_pipeline_stall_queue_depth` increases while pod CPU stays below 40 percent; suspect downstream refusal, replay backlog, or feature flag deadlock.
- Tenant-specific shape: one `tenant_id` dominates labels in `payments_kyc_aml_screening_pipeline_stall_queue_depth`; isolate tenant before fleet mitigation.
- Fleet-wide shape: at least three cells report `PaymentsKYCAMLScreeningPipelineStallCritical` in one 15 minute window; switch to cross-cell bridge even if individual tenants are low-volume.
- Log signature `decision=deny reason=kyc-aml-screening-pipeline-stall.policy_guard` means the guard is working; investigate caller inputs before rollback.
- Log signature `decision=permit reason=kyc-aml-screening-pipeline-stall.break_glass` means manual intervention is active; confirm two-person authorization.
- Log signature `audit_emit_status=stalled event_class=EVT_PAYMENTS_KYC_AML_SCREENING_PIPELINE_STALL_INCIDENT` means mitigation cannot be closed until replay succeeds.
- Service-specific metric pattern: `payments_kyc_aml_screening_pipeline_stall_ledger_mismatch_total` rises while `payments_kyc_aml_screening_pipeline_stall_aml_review_lag_seconds` is flat; inspect local worker health before escalating vendors.
- Service-specific metric pattern: `payments_kyc_aml_screening_pipeline_stall_aml_review_lag_seconds` rises while `payments_kyc_aml_screening_pipeline_stall_error_ratio` is flat; suspect stale export, stale recommendation, stale projection, or vendor dependency lag.

## Diagnostic Steps
1. Set incident variables: `export INCIDENT_ID=INC-payments-kyc-aml-screening-pipeline-stall-$(date -u +%Y%m%dT%H%M%SZ); export CELL=prod-us-east-1; export TENANT=synthetic-canary`.
2. Confirm active alerts: `curl -s https://payments.internal.oyatie.dev/v1/payments/alerts?runbook=kyc-aml-screening-pipeline-stall | jq .alerts`.
3. Check Kubernetes rollout: `kubectl -n payments rollout status deploy/payments-kyc-aml-screening-pipeline-stall-worker --timeout=60s`.
4. List unhealthy pods: `kubectl -n payments get pods -l app=kyc-aml-screening-pipeline-stall -o wide`.
5. Read structured logs: `kubectl -n payments logs deploy/payments-kyc-aml-screening-pipeline-stall-worker --since=30m | rg "payments.kyc_aml_screening_pipeline_stall.incident_state|PaymentsKYCAMLScreeningPipelineStallCritical|EVT_PAYMENTS_KYC_AML_SCREENING_PIPELINE_STALL_INCIDENT"`.
6. Query Loki directly: `logcli query '{namespace="payments",runbook="kyc-aml-screening-pipeline-stall"}' --since=30m --limit=200`.
7. Check Prometheus error ratio: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=payments_kyc_aml_screening_pipeline_stall_error_ratio{cell="prod-us-east-1"}'`.
8. Check lag: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=payments_kyc_aml_screening_pipeline_stall_lag_seconds{cell="prod-us-east-1"}'`.
9. Check queue: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=payments_kyc_aml_screening_pipeline_stall_queue_depth{cell="prod-us-east-1"}'`.
10. Check service-specific signal: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=payments_kyc_aml_screening_pipeline_stall_psp_decline_ratio{cell="prod-us-east-1"}'`.
11. Open primary dashboard: `open "https://grafana.dev.oyatie.internal/d/payments-substrate/kyc-aml-screening-pipeline-stall?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=116&var-incident=$INCIDENT_ID"`.
12. Open secondary dashboard: `open "https://grafana.dev.oyatie.internal/d/payments-substrate/kyc-aml-screening-pipeline-stall?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=213&var-tenant=$TENANT"`.
13. Verify audit-chain emission: `oya audit-chain query --event-class EVT_PAYMENTS_KYC_AML_SCREENING_PIPELINE_STALL_INCIDENT --since 30m --cell $CELL --tenant $TENANT`.
14. Verify service state: `oya ops payments kyc-aml-screening-pipeline-stall status --cell $CELL --tenant $TENANT --output json`.
15. Run production snapshot gate: `cargo run -p dev-cli -- gate validate payments-kyc-aml-screening-pipeline-stall --production-snapshot --cell $CELL`.
16. Run payments contract smoke: `cargo run -p dev-cli -- gate validate payments-contract --microservice payments --scenario kyc-aml-screening-pipeline-stall`.
17. Check API contract smoke: `curl -s https://payments.internal.oyatie.dev/v1/payments/kyc-aml-screening-pipeline-stall/incident-handoff -H "x-tenant: $TENANT"`.
18. Inspect config: `test -f microservices/payments/iac/kustomize/base/kustomization.yaml && sed -n '1,180p' microservices/payments/iac/kustomize/base/kustomization.yaml`.
19. Inspect feature flags: `oya flags get oya.payments.kyc_aml_screening_pipeline_stall.incident_hold --cell $CELL --tenant $TENANT --output yaml`.
20. Inspect circuit breaker: `oya ops breaker status payments-kyc-aml-screening-pipeline-stall-circuit-breaker --cell $CELL --tenant $TENANT`.
21. Check recent deploy: `kubectl -n payments rollout history deploy/payments-kyc-aml-screening-pipeline-stall-worker | tail -20`.
22. Check policy file: `test -f microservices/payments/policy/charge-authorization.cedar || find microservices/payments/policy -maxdepth 2 -type f | sort`.
23. Check SLO files: `ls microservices/payments/slos/*.openslo.yaml | sort | rg "charge|kyc"`.
24. Check catalog components: `find microservices/payments/catalog -maxdepth 1 -type f | sort | rg "charge|refund|dispute|payout|settlement|kyc|subscription|adapter"`.
25. Run targeted SQL state query: `psql $OYATIE_PROD_DSN -c "select incident_id, tenant_id, cell_id, state, updated_at from payments_kyc_aml_screening_pipeline_stall_incidents where updated_at > now() - interval '30 minutes' order by updated_at desc limit 20;"`.
26. Confirm no cross-cell spread: `oya ops cells query --metric payments_kyc_aml_screening_pipeline_stall_error_ratio --window 30m --threshold 0.02`.
27. Snapshot evidence: `oya evidence snapshot --incident $INCIDENT_ID --microservice payments --runbook kyc-aml-screening-pipeline-stall --output evidence/incidents/$INCIDENT_ID.json`.

### Diagnostic Decision Tree
```text
KYC AML Screening Pipeline Stall incident decision tree
1. Is PaymentsKYCAMLScreeningPipelineStallCritical firing in more than one cell?
   |-- yes: declare fleet incident, page PagerDuty payments-primary; fraud-risk secondary, and run cross-cell containment.
   |-- no: keep scope to the affected cell and continue tenant isolation checks.
2. Does payments_kyc_aml_screening_pipeline_stall_queue_depth grow while payments_kyc_aml_screening_pipeline_stall_error_ratio is flat?
   |-- yes: downstream dependency, replay backlog, or queue-drain issue; choose mitigation branch B.
   |-- no: local regression, bad input, or policy/config drift; continue branch selection.
3. Does audit-chain show EVT_PAYMENTS_KYC_AML_SCREENING_PIPELINE_STALL_INCIDENT gaps?
   |-- yes: do not close; run evidence replay before resolution.
   |-- no: mitigation can proceed after state is green.
4. Is customer, finance, security, or regulator impact confirmed?
   |-- yes: promote severity, open #inc-payments, and notify compliance or security handoff.
   |-- no: keep internal incident and collect evidence.
```
- Branch A (confirmed money movement correctness risk): use the matching mitigation block below and record `decision_branch=A` in `EVT_PAYMENTS_KYC_AML_SCREENING_PIPELINE_STALL_INCIDENT`.
- Branch B (PSP dependency degraded): use the matching mitigation block below and record `decision_branch=B` in `EVT_PAYMENTS_KYC_AML_SCREENING_PIPELINE_STALL_INCIDENT`.
- Branch C (fraud or AML safety risk): use the matching mitigation block below and record `decision_branch=C` in `EVT_PAYMENTS_KYC_AML_SCREENING_PIPELINE_STALL_INCIDENT`.
- Branch D (customer-visible financial impact): use the matching mitigation block below and record `decision_branch=D` in `EVT_PAYMENTS_KYC_AML_SCREENING_PIPELINE_STALL_INCIDENT`.

## Mitigation Steps
1. Acknowledge page: `pd incident ack --service payments --incident $INCIDENT_ID`.
2. Create bridge: `oya incident bridge create --incident $INCIDENT_ID --channel #inc-payments --severity sev0`.
3. Freeze risky automation: `oya flags set oya.payments.kyc_aml_screening_pipeline_stall.incident_hold=true --cell $CELL --tenant $TENANT --reason $INCIDENT_ID`.
4. Enable circuit breaker: `oya ops breaker open payments-kyc-aml-screening-pipeline-stall-circuit-breaker --cell $CELL --tenant $TENANT --ttl 30m --reason $INCIDENT_ID`.
5. Reduce blast radius: `kubectl -n payments scale deploy/payments-kyc-aml-screening-pipeline-stall-worker --replicas=1`.
6. Protect tenant boundary: `oya tenancy quarantine --tenant $TENANT --reason payments-kyc-aml-screening-pipeline-stall --ttl 60m`.
7. Pause promotion: incident hold PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
8. Drain queue safely: `oya ops payments kyc-aml-screening-pipeline-stall drain --cell $CELL --tenant $TENANT --max-items 500 --dry-run`.
9. Execute bounded drain: `oya ops payments kyc-aml-screening-pipeline-stall drain --cell $CELL --tenant $TENANT --max-items 500 --confirm $INCIDENT_ID`.
10. Replay missing audit events: `oya audit-chain replay --event-class EVT_PAYMENTS_KYC_AML_SCREENING_PIPELINE_STALL_INCIDENT --incident $INCIDENT_ID --from evidence/incidents/$INCIDENT_ID.json`.
11. Rollback last deploy if causal: `kubectl -n payments rollout undo deploy/payments-kyc-aml-screening-pipeline-stall-worker`.
12. Raise HPA cap if saturation is proven: `kubectl -n payments patch hpa payments-kyc-aml-screening-pipeline-stall-worker --type merge -p '{"spec":{"maxReplicas":12}}'`.
13. Throttle hot tenant: `oya ops rate-limit set --tenant $TENANT --surface payments.kyc-aml-screening-pipeline-stall --rps 25 --ttl 30m`.
14. Block abusive principal when relevant: `oya identity principal suspend --principal suspected-abuse --tenant $TENANT --reason $INCIDENT_ID`.
15. Protect evidence: `oya evidence freeze --incident $INCIDENT_ID --paths microservices/payments/runbooks/kyc-aml-screening-pipeline-stall.md,evidence/incidents/$INCIDENT_ID.json`.
16. Notify service owners: `oya notify service-owner --microservice payments --incident $INCIDENT_ID --channel #inc-payments`.
17. Open external vendor ticket: `oya vendor ticket open --vendor "Stripe enterprise support" --incident $INCIDENT_ID --summary payments-kyc-aml-screening-pipeline-stall`.
18. Confirm breaker effect: `oya ops breaker status payments-kyc-aml-screening-pipeline-stall-circuit-breaker --cell $CELL --tenant $TENANT --expect open`.
19. Confirm user impact reduced: `curl -s https://payments.internal.oyatie.dev/v1/payments/kyc-aml-screening-pipeline-stall/incident-handoff/health -H "x-tenant: $TENANT"`.
20. Emit mitigation audit: `oya audit-chain emit --event-class EVT_PAYMENTS_KYC_AML_SCREENING_PIPELINE_STALL_INCIDENT --incident $INCIDENT_ID --field mitigation=active --field runbook=kyc-aml-screening-pipeline-stall`.

### Mitigation Branch Guidance
- Branch A: confirmed money movement correctness risk.
  - Required action: keep `payments-kyc-aml-screening-pipeline-stall-circuit-breaker` open until `payments_kyc_aml_screening_pipeline_stall_error_ratio` is below 0.005 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/payments-substrate/kyc-aml-screening-pipeline-stall?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=117` to the incident.
  - Required audit: emit `EVT_PAYMENTS_KYC_AML_SCREENING_PIPELINE_STALL_INCIDENT` with `branch=A`, `operator_id`, and `evidence_hash`.
- Branch B: PSP dependency degraded.
  - Required action: keep `payments-kyc-aml-screening-pipeline-stall-circuit-breaker` open until `payments_kyc_aml_screening_pipeline_stall_error_ratio` is below 0.01 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/payments-substrate/kyc-aml-screening-pipeline-stall?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=118` to the incident.
  - Required audit: emit `EVT_PAYMENTS_KYC_AML_SCREENING_PIPELINE_STALL_INCIDENT` with `branch=B`, `operator_id`, and `evidence_hash`.
- Branch C: fraud or AML safety risk.
  - Required action: keep `payments-kyc-aml-screening-pipeline-stall-circuit-breaker` open until `payments_kyc_aml_screening_pipeline_stall_error_ratio` is below 0.005 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/payments-substrate/kyc-aml-screening-pipeline-stall?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=119` to the incident.
  - Required audit: emit `EVT_PAYMENTS_KYC_AML_SCREENING_PIPELINE_STALL_INCIDENT` with `branch=C`, `operator_id`, and `evidence_hash`.
- Branch D: customer-visible financial impact.
  - Required action: keep `payments-kyc-aml-screening-pipeline-stall-circuit-breaker` open until `payments_kyc_aml_screening_pipeline_stall_error_ratio` is below 0.01 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/payments-substrate/kyc-aml-screening-pipeline-stall?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=120` to the incident.
  - Required audit: emit `EVT_PAYMENTS_KYC_AML_SCREENING_PIPELINE_STALL_INCIDENT` with `branch=D`, `operator_id`, and `evidence_hash`.

## Resolution Steps
1. Identify code owner path: `rg "kyc_aml_screening_pipeline_stall|PaymentsKYCAMLScreeningPipelineStallCritical|payments.kyc_aml_screening_pipeline_stall.incident_state" crates microservices/payments -g "!microservices/payments/runbooks/**"`.
2. Patch catalog invariant: `edit microservices/payments/catalog/payments-charge-domain.yaml or the matching refund/dispute/payout/KYC catalog record where kyc_aml_screening_pipeline_stall state transition is validated`.
3. Patch API guard: `edit microservices/payments/contracts/openapi-v1.yaml if the failing path is north-south or async handoff`.
4. Patch policy: `edit microservices/payments/policy/charge-authorization.cedar with explicit deny/permit branch and tenant/cell scope`.
5. Patch runtime config: `edit microservices/payments/iac/kustomize/base/kustomization.yaml if deploy/config drift caused the incident`.
6. Add regression gate: `cargo run -p dev-cli -- gate validate payments-contract --fixture incident-kyc-aml-screening-pipeline-stall.json`.
7. Add gate evidence: `cargo run -p dev-cli -- gate validate payments-kyc-aml-screening-pipeline-stall --fixture incident-kyc-aml-screening-pipeline-stall.json`.
8. Add SLO assertion: `update microservices/payments/slos/charge-api-availability.openslo.yaml with alert PaymentsKYCAMLScreeningPipelineStallCritical when this was a missing alert`.
9. Add dashboard panel: `update microservices/payments/dashboards/psp-routing.json with payments_kyc_aml_screening_pipeline_stall_error_ratio, payments_kyc_aml_screening_pipeline_stall_lag_seconds, and payments_kyc_aml_screening_pipeline_stall_queue_depth`.
10. Rebuild validation CLI: `cargo check -p dev-cli --all-targets`.
11. Run targeted validation: `cargo run -p dev-cli -- gate validate payments-policy --microservice payments`.
12. Run policy validation: `cargo run -p dev-cli -- gate validate payments-policy --microservice payments`.
13. Deploy canary: `oya deploy canary --microservice payments --component payments-kyc-aml-screening-pipeline-stall-worker --cell $CELL --weight 1`.
14. Watch burn rate: `oya ops watch --metric payments_kyc_aml_screening_pipeline_stall_error_ratio --threshold 0.005 --window 30m --cell $CELL`.
15. Close circuit breaker: `oya ops breaker close payments-kyc-aml-screening-pipeline-stall-circuit-breaker --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
16. Unfreeze automation: `oya flags set oya.payments.kyc_aml_screening_pipeline_stall.incident_hold=false --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
17. Resume promotion: recovery PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
18. Seal resolution audit: `oya audit-chain emit --event-class EVT_PAYMENTS_KYC_AML_SCREENING_PIPELINE_STALL_INCIDENT --incident $INCIDENT_ID --field resolution=complete --field runbook=kyc-aml-screening-pipeline-stall`.
19. Verify seal: `oya audit-chain verify --event-class EVT_PAYMENTS_KYC_AML_SCREENING_PIPELINE_STALL_INCIDENT --incident $INCIDENT_ID`.
20. Attach final evidence: `oya evidence attach --incident $INCIDENT_ID --file evidence/incidents/$INCIDENT_ID.json --kind final-resolution`.

### Code Paths To Inspect First
- `microservices/payments/catalog/payments-charge-domain.yaml`: inspect for `kyc_aml_screening_pipeline_stall` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `microservices/payments/catalog/payments-refund-domain.yaml`: inspect for `kyc_aml_screening_pipeline_stall` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `microservices/payments/catalog/payments-dispute-domain.yaml`: inspect for `kyc_aml_screening_pipeline_stall` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `microservices/payments/catalog/payments-payout-domain.yaml`: inspect for `kyc_aml_screening_pipeline_stall` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `microservices/payments/contracts/openapi-v1.yaml`: verify request/response or event contract only when incident evidence points there.
- `microservices/payments/contracts/asyncapi-v1.yaml`: verify request/response or event contract only when incident evidence points there.
- `microservices/payments/contracts/payments-v1.proto`: verify request/response or event contract only when incident evidence points there.
- `microservices/payments/contracts/psp-adapter-trait.md`: verify request/response or event contract only when incident evidence points there.
- `microservices/payments/dashboards/psp-routing.json`: verify panel coverage for `payments_kyc_aml_screening_pipeline_stall_error_ratio`, `payments_kyc_aml_screening_pipeline_stall_lag_seconds`, and `payments_kyc_aml_screening_pipeline_stall_psp_decline_ratio`.
- `microservices/payments/slos/`: verify alert vocabulary and threshold alignment before changing runtime thresholds.
- `microservices/payments/policy/`: verify policy branch ownership before relaxing deny rules or emergency bypasses.

## Verification Checklist
- `PaymentsKYCAMLScreeningPipelineStallCritical` and `PaymentsKYCAMLScreeningPipelineStallSloBurn` are both resolved in Alertmanager for 30 minutes.
- `payments_kyc_aml_screening_pipeline_stall_error_ratio < 0.005` for 3 consecutive 10 minute windows.
- `payments_kyc_aml_screening_pipeline_stall_lag_seconds < 120` for all production cells.
- `payments_kyc_aml_screening_pipeline_stall_queue_depth` is draining and not growing for the affected tenant.
- Service-specific signal `payments_kyc_aml_screening_pipeline_stall_psp_decline_ratio` is below the threshold documented in `microservices/payments/slos/charge-api-availability.openslo.yaml`.
- dashboard `https://grafana.dev.oyatie.internal/d/payments-substrate/kyc-aml-screening-pipeline-stall?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=116` shows green panels for the affected cell.
- audit-chain query for `EVT_PAYMENTS_KYC_AML_SCREENING_PIPELINE_STALL_INCIDENT` returns mitigation and resolution events.
- circuit breaker `payments-kyc-aml-screening-pipeline-stall-circuit-breaker` is closed after rollback window.
- feature flag `oya.payments.kyc_aml_screening_pipeline_stall.incident_hold` is false for the affected tenant unless long-term hold is approved.
- runbook invocation evidence is attached to `evidence/incidents/$INCIDENT_ID.json`.
- service owner acknowledged final handoff in `#inc-payments`.

## Postmortem Template
Use this exact skeleton for the incident document. The field names are intentionally stable for ADR-0263 audit emission extraction.
```markdown
---
doc_class: IncidentPostmortem
runbook_id: payments-kyc-aml-screening-pipeline-stall
microservice: payments
event_class: EVT_PAYMENTS_KYC_AML_SCREENING_PIPELINE_STALL_INCIDENT
incident_id: <INC-...>
severity: sev0
status: draft
detected_at: <UTC>
mitigated_at: <UTC>
resolved_at: <UTC>
commander: <handle>
evidence_hash: <sha256>
---

# KYC AML Screening Pipeline Stall postmortem

## Summary
- What happened in payments/kyc-aml-screening-pipeline-stall.
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
- Emit EVT_PAYMENTS_KYC_AML_SCREENING_PIPELINE_STALL_INCIDENT with incident_id, tenant_id, cell_id, principal_id, decision_id, evidence_hash, operator_id, runbook_id.
- Attach dashboard snapshot URLs and command transcripts.
- Seal mitigation and resolution events before closure.

## Corrective Actions
- Action, owner, due date, validation command, linked issue.
```

## Escalation Path
- Primary on-call: PagerDuty payments-primary; fraud-risk secondary.
- Incident SLA: ack 3m for sev0/sev1, 10m for sev2, 30m for sev3; status update every 10m until `PaymentsKYCAMLScreeningPipelineStallCritical` clears.
- Incident commander: first responder from axis-payments + ops-sre-reliability + ops-security; transfer only by explicit message in `#inc-payments`.
- Security escalation: page `ops-security-primary` immediately for sev0, credential, cross-tenant, fraud, or audit-seal symptoms.
- Compliance escalation: page `dpo-office-duty` when tenant data, regulator evidence, money movement, or breach-clock symptoms are present.
- Architecture escalation: page `council-architecture-reviewer` before manual bypass, policy rollback, or invariant relaxation.
- External vendors: Stripe enterprise support; Adyen technical support; Visa DPS risk support; Korean FSS liaison desk. Open a ticket once local dependency health is proven and vendor dependency remains suspect.
- Customer communications: use status page component `oyatie-payments-kyc-aml-screening-pipeline-stall` and keep private details in the incident channel.
- Regulatory clock: if tenant data, financial correctness, or evidence integrity is possibly affected, start the compliance 72h assessment timer even if exposure is unconfirmed.
- Executive notice: sev0 or fleet-wide sev1 goes to `#exec-incident-readout` within 30 minutes.

## Cross-µservice Coordination
- Notify `identity`: `oya incident handoff --target identity --source payments --runbook kyc-aml-screening-pipeline-stall --incident $INCIDENT_ID --severity sev0 --branch A`; expect `202 accepted`.
- Require `identity` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `finops-portal`: `oya incident handoff --target finops-portal --source payments --runbook kyc-aml-screening-pipeline-stall --incident $INCIDENT_ID --severity sev0 --branch B`; expect `202 accepted`.
- Require `finops-portal` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `audit-chain`: `oya incident handoff --target audit-chain --source payments --runbook kyc-aml-screening-pipeline-stall --incident $INCIDENT_ID --severity sev0 --branch C`; expect `202 accepted`.
- Require `audit-chain` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `compliance`: `oya incident handoff --target compliance --source payments --runbook kyc-aml-screening-pipeline-stall --incident $INCIDENT_ID --severity sev0 --branch D`; expect `202 accepted`.
- Require `compliance` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Observability handoff API: `oya incident handoff --target observability --source payments --runbook kyc-aml-screening-pipeline-stall --incident $INCIDENT_ID`.
- Governance handoff API: `oya incident handoff --target governance --source payments --runbook kyc-aml-screening-pipeline-stall --incident $INCIDENT_ID`.
- Compliance handoff API: `oya incident handoff --target compliance --source payments --runbook kyc-aml-screening-pipeline-stall --incident $INCIDENT_ID`.
- Audit-chain handoff API: `oya incident handoff --target audit-chain --source payments --runbook kyc-aml-screening-pipeline-stall --incident $INCIDENT_ID`.
- Tenancy handoff API: `oya incident handoff --target tenancy --source payments --runbook kyc-aml-screening-pipeline-stall --incident $INCIDENT_ID`.

## Handoff Notes
- Do not hand off with only the alert name; include `payments_kyc_aml_screening_pipeline_stall_error_ratio`, `payments_kyc_aml_screening_pipeline_stall_lag_seconds`, `payments_kyc_aml_screening_pipeline_stall_queue_depth`, `payments_kyc_aml_screening_pipeline_stall_psp_decline_ratio`, current breaker state, and audit seal status.
- Keep `payments-kyc-aml-screening-pipeline-stall-circuit-breaker` owner as axis-payments + ops-sre-reliability + ops-security until the receiving service explicitly accepts.
- If another runbook owns the downstream fix, link this incident as upstream and keep this runbook open until downstream verification returns green.
- Close only after `EVT_PAYMENTS_KYC_AML_SCREENING_PIPELINE_STALL_INCIDENT` has a sealed resolution row and every coordination endpoint above has either accepted or explicitly declined scope.

## Sources Checked During This Substance Pass
- `microservices/payments/dashboards/` for dashboard names and operational panels: psp-routing.json, settlement-reconciliation.json, fraud-signals.md, dispute-volume.json.
- `microservices/payments/slos/` for OpenSLO alert vocabulary and threshold alignment: charge-api-availability.openslo.yaml, refund-api-availability.openslo.yaml, payout-completion-success.openslo.yaml, dispute-response-latency.openslo.yaml.
- `microservices/payments/policy/` for named policy and authorization surfaces: charge-authorization.cedar, refund-authorization.cedar, payout-authorization.cedar, dispute-authorization.cedar, abuse-defence.cedar.
- `microservices/payments/contracts/` for API, AsyncAPI, proto, and adapter surfaces: contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/payments-v1.proto, contracts/psp-adapter-trait.md.
- `microservices/payments/catalog/` for component and owner vocabulary; existing runbook topic `kyc-aml-screening-pipeline-stall` was preserved as the scenario anchor.
