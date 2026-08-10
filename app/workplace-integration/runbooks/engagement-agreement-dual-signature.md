---
doc_class: Runbook
title: Engagement Agreement Dual Signature
status: Accepted
date: 2026-05-20
microservice: workplace-integration
severity: sev1
audience: workplace-incident-commander
owner_team: axis-workplace-integration + ops-sre-reliability
source_wave: codex-runbooks-substrate-w3
change_scope: substance rewrite of existing thin runbook
doc_status: draft_target_non_claim
---

# Runbook: Engagement Agreement Dual Signature

## Operator Contract
- Runbook id: workplace-integration-engagement-agreement-dual-signature.
- Primary service namespace: `workplace-integration`.
- Owning rotation: PagerDuty oya-workplace-integration-primary; hris-esign-secondary.
- Incident channel: `#inc-workplace-integration`.
- Operational focus: dual signature agreement has one side complete and the other blocked.
- Named precedent: this follows the DocuSign envelope recovery plus Workday roster reconciliation pattern.
- External dependencies: DocuSign enterprise support; Workday HCM support; ADP Workforce Now support.
- API authority: `https://workplace-integration.internal.oyatie.dev/v1/workplace-integration/engagement-agreement-dual-signature/incident-handoff`.
- Audit event class: `EVT_WORKPLACE_INTEGRATION_ENGAGEMENT_AGREEMENT_DUAL_SIGNATURE_INCIDENT` with ADR-0263 fields `incident_id`, `tenant_id`, `cell_id`, `microservice`, `runbook_id`, `decision_id`, `evidence_hash`, `operator_id`.
- Stop condition: mitigation has held for 30 minutes, `EngagementAgreementDualSignatureCritical` is green, and every Cross-microservice handoff API returns `202 accepted`.
- Safety invariant: never clear the incident until `EVT_WORKPLACE_INTEGRATION_ENGAGEMENT_AGREEMENT_DUAL_SIGNATURE_INCIDENT` is sealed and the postmortem skeleton exists under `evidence/postmortems/workplace-integration-engagement-agreement-dual-signature-<incident-id>.md`.

## Trigger Conditions
- Page on alert `EngagementAgreementDualSignatureCritical` when `oya_workplace_integration_engagement_agreement_dual_signature_error_ratio > 0.02` for 10 minutes in any production cell.
- Page on alert `EngagementAgreementDualSignatureSloBurn` when `oya_workplace_integration_engagement_agreement_dual_signature_lag_seconds > 300` for 2 consecutive evaluator windows.
- Open sev1 if `oya_workplace_integration_dual_signature_stall_total` exceeds the threshold documented in `app/workplace-integration/slos/clock-attestation-availability.openslo.yaml`.
- Open sev1 if `oya_workplace_integration_engagement_agreement_dual_signature_queue_depth > 5000` for 15 minutes or retry backlog grows by more than 20 percent in one 5 minute window.
- Trigger from customer report when Support tags the case `workplace-integration.engagement-agreement-dual-signature.customer_visible` in Zendesk.
- Trigger from `oya-ci-required` when the workplace-integration `engagement-agreement-dual-signature` production-snapshot Rust gate exits non-zero against the latest production evidence bundle.
- Primary dashboard: `https://grafana.dev.oyatie.internal/d/workplace-integration-ops/engagement-agreement-dual-signature?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=101` backed by `app/workplace-integration/dashboards/audit-evidence.json`.
- Secondary dashboard: `https://grafana.dev.oyatie.internal/d/workplace-integration-ops/engagement-agreement-dual-signature?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=202` backed by `app/workplace-integration/dashboards/policy-deny-rate.json`.
- Loki explorer: `https://grafana.dev.oyatie.internal/explore?query={namespace="workplace-integration",runbook="engagement-agreement-dual-signature"}`.
- Alertmanager route: `oyatie-workplace-integration-engagement-agreement-dual-signature-critical`; silence only with incident commander approval and `EVT_WORKPLACE_INTEGRATION_ENGAGEMENT_AGREEMENT_DUAL_SIGNATURE_INCIDENT` evidence.
- Synthetic probe: `oya ops probe workplace-integration engagement-agreement-dual-signature --cell prod-us-east-1 --tenant synthetic-canary` returns `healthy=true`.
- Drift detector: `registry/workplace-integration/engagement-agreement-dual-signature/expected-state.json` hash differs from live `https://workplace-integration.internal.oyatie.dev/v1/workplace-integration/engagement-agreement-dual-signature/admin/state-hash`.
- Service-specific metric `oya_workplace_integration_dual_signature_stall_total` is red while `oya_workplace_integration_engagement_agreement_dual_signature_audit_emit_total{status="sealed"}` is flat.

## Symptoms
- User-facing impact: employees, candidates, or workforce admins may see broken clock, e-sign, roster, offer, or compliance handoffs; scenario focus is dual signature agreement has one side complete and the other blocked.
- Operators see Grafana panel `audit-evidence.json / Engagement Agreement Dual Signature burn rate` turn red before the primary alert resolves.
- Loki signature `workplace_integration.engagement_agreement_dual_signature.incident_state=failed` appears with fields `incident_id`, `tenant_id`, `cell_id`, `decision_id`, `evidence_hash`.
- Kubernetes events include `reason=EngagementAgreementDualSignatureDegraded` on deployment `workplace-integration-engagement-agreement-dual-signature-worker` or `workplace-integration-api`.
- Audit-chain shows missing or delayed `EVT_WORKPLACE_INTEGRATION_ENGAGEMENT_AGREEMENT_DUAL_SIGNATURE_INCIDENT` entries when queried with `oya audit-chain query --event-class EVT_WORKPLACE_INTEGRATION_ENGAGEMENT_AGREEMENT_DUAL_SIGNATURE_INCIDENT --since 30m`.
- Metric pattern: `oya_workplace_integration_engagement_agreement_dual_signature_error_ratio` rises before `oya_workplace_integration_engagement_agreement_dual_signature_lag_seconds`; if lag rises first, suspect dependency saturation rather than local regression.
- Metric pattern: `oya_workplace_integration_engagement_agreement_dual_signature_queue_depth` increases while pod CPU stays below 40 percent; suspect downstream refusal, replay backlog, or feature flag deadlock.
- Tenant-specific shape: one `tenant_id` dominates labels in `oya_workplace_integration_engagement_agreement_dual_signature_queue_depth`; isolate tenant before fleet mitigation.
- Fleet-wide shape: at least three cells report `EngagementAgreementDualSignatureCritical` in one 15 minute window; switch to cross-cell bridge even if individual tenants are low-volume.
- Log signature `decision=deny reason=engagement-agreement-dual-signature.policy_guard` means the guard is working; investigate caller inputs before rollback.
- Log signature `decision=permit reason=engagement-agreement-dual-signature.break_glass` means manual intervention is active; confirm two-person authorization.
- Log signature `audit_emit_status=stalled event_class=EVT_WORKPLACE_INTEGRATION_ENGAGEMENT_AGREEMENT_DUAL_SIGNATURE_INCIDENT` means mitigation cannot be closed until replay succeeds.
- Service-specific pattern: `oya_workplace_integration_dual_signature_stall_total` rises while `oya_workplace_integration_engagement_agreement_dual_signature_dependency_error_ratio` is flat; inspect local state before escalating DocuSign enterprise support.
- Service-specific pattern: `oya_workplace_integration_engagement_agreement_dual_signature_dependency_error_ratio` rises while `oya_workplace_integration_dual_signature_stall_total` is flat; inspect vendor or adjacent-service dependency health before local rollback.

## Failure Mode Tree
- Failure mode 1: single-tenant WorkplaceAgreement inconsistency; contain with tenant quarantine, preserve all `EVT_WORKPLACE_INTEGRATION_ENGAGEMENT_AGREEMENT_DUAL_SIGNATURE_INCIDENT` rows, and avoid fleet rollback.
- Failure mode 2: cross-cell ESignSession drift; freeze writes, compare state hash across cells, and use audit-chain replay before accepting new mutations.
- Failure mode 3: byzantine or abusive principal; suspend the principal through identity, keep tenant data scoped, and preserve Cedar explain output.
- Failure mode 4: external dependency outage at DocuSign enterprise support; open vendor ticket only after local dashboards and handoff APIs prove the dependency is causal.
- Failure mode 5: operator mitigation made state worse; roll back feature flag `oya.workplace-integration.engagement_agreement_dual_signature.incident_hold`, close `workplace-integration-engagement-agreement-dual-signature-circuit-breaker`, and restore the previous deployment revision.
- Failure mode 6: audit emission is delayed; do not close even when customer symptoms improve because ADR-0263 evidence is incomplete.
- Failure mode 7: regional partition; keep prod-us-east-1 as evidence leader and reject cross-region mutation until `oya_workplace_integration_engagement_agreement_dual_signature_state_hash_match == 1`.
- Failure mode 8: compliance-pack mismatch; require compliance handoff when KR-CSAP, EU-sovereign, FedRAMP-High, IL5, or CN-PIPL labels are present.
- Failure mode 9: stale dashboard data; verify direct Mimir queries before making rollback decisions.
- Failure mode 10: runbook step ambiguity; halt the ambiguous branch, emit `EVT_WORKPLACE_INTEGRATION_ENGAGEMENT_AGREEMENT_DUAL_SIGNATURE_INCIDENT` with outcome `blocked`, and patch this runbook after recovery.

## Diagnostic Steps
1. Set incident variables: `export INCIDENT_ID=INC-workplace-integration-engagement-agreement-dual-signature-$(date -u +%Y%m%dT%H%M%SZ); export CELL=prod-us-east-1; export TENANT=synthetic-canary`.
2. Confirm active alerts: `curl -s https://workplace-integration.internal.oyatie.dev/v1/workplace-integration/alerts?runbook=engagement-agreement-dual-signature | jq .alerts`.
3. Check Kubernetes rollout: `kubectl -n workplace-integration rollout status deploy/workplace-integration-engagement-agreement-dual-signature-worker --timeout=60s`.
4. List unhealthy pods: `kubectl -n workplace-integration get pods -l app=engagement-agreement-dual-signature -o wide`.
5. Read structured logs: `kubectl -n workplace-integration logs deploy/workplace-integration-engagement-agreement-dual-signature-worker --since=30m | rg "workplace_integration.engagement_agreement_dual_signature.incident_state|EngagementAgreementDualSignatureCritical|EVT_WORKPLACE_INTEGRATION_ENGAGEMENT_AGREEMENT_DUAL_SIGNATURE_INCIDENT"`.
6. Query Loki directly: `logcli query '{namespace="workplace-integration",runbook="engagement-agreement-dual-signature"}' --since=30m --limit=200`.
7. Check Prometheus error ratio: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_workplace_integration_engagement_agreement_dual_signature_error_ratio{cell="prod-us-east-1"}'`.
8. Check lag: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_workplace_integration_engagement_agreement_dual_signature_lag_seconds{cell="prod-us-east-1"}'`.
9. Check queue: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_workplace_integration_engagement_agreement_dual_signature_queue_depth{cell="prod-us-east-1"}'`.
10. Check service-specific signal: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_workplace_integration_dual_signature_stall_total{cell="prod-us-east-1"}'`.
11. Open primary dashboard: `open "https://grafana.dev.oyatie.internal/d/workplace-integration-ops/engagement-agreement-dual-signature?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=101&var-incident=$INCIDENT_ID"`.
12. Open secondary dashboard: `open "https://grafana.dev.oyatie.internal/d/workplace-integration-ops/engagement-agreement-dual-signature?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=202&var-tenant=$TENANT"`.
13. Verify audit-chain emission: `oya audit-chain query --event-class EVT_WORKPLACE_INTEGRATION_ENGAGEMENT_AGREEMENT_DUAL_SIGNATURE_INCIDENT --since 30m --cell $CELL --tenant $TENANT`.
14. Verify service state: `oya ops workplace-integration engagement-agreement-dual-signature status --cell $CELL --tenant $TENANT --output json`.
15. Attach branch-protected CI evidence: `oya-ci-required` shared Rust gate evidence.
16. Run Buck target smoke check: `buck2 test root//app/workplace-integration/crates/oya-workplace-integration-doc-set-scaffold:oya-workplace-integration-doc-set-scaffold`.
17. Check API contract smoke: `curl -s https://workplace-integration.internal.oyatie.dev/v1/workplace-integration/engagement-agreement-dual-signature/incident-handoff -H "x-oya-tenant: $TENANT"`.
18. Inspect config: `test -f app/workplace-integration/iac/kustomize/base/kustomization.yaml && sed -n '1,180p' app/workplace-integration/iac/kustomize/base/kustomization.yaml`.
19. Inspect feature flags: `oya flags get oya.workplace-integration.engagement_agreement_dual_signature.incident_hold --cell $CELL --tenant $TENANT --output yaml`.
20. Inspect circuit breaker: `oya ops breaker status workplace-integration-engagement-agreement-dual-signature-circuit-breaker --cell $CELL --tenant $TENANT`.
21. Check recent deploy: `kubectl -n workplace-integration rollout history deploy/workplace-integration-engagement-agreement-dual-signature-worker | tail -20`.
22. Check policy file: `test -f app/workplace-integration/policies/esign-initiate.cedar || find app/workplace-integration/policy -maxdepth 2 -type f | sort`.
23. Check SLO files: `ls app/workplace-integration/slos/*.openslo.yaml | sort | rg "clock|dlp"`.
24. Check contract binding: `test -f app/workplace-integration/contracts/openapi-v1.yaml && sed -n '1,120p' app/workplace-integration/contracts/openapi-v1.yaml`.
25. Run targeted SQL state query: `psql $OYA_PROD_DSN -c "select incident_id, tenant_id, cell_id, state, updated_at from workplace_integration_engagement_agreement_dual_signature_incidents where updated_at > now() - interval '30 minutes' order by updated_at desc limit 20;"`.
26. Confirm no cross-cell spread: `oya ops cells query --metric oya_workplace_integration_engagement_agreement_dual_signature_error_ratio --window 30m --threshold 0.02`.
27. Snapshot evidence: `oya evidence snapshot --incident $INCIDENT_ID --microservice workplace-integration --runbook engagement-agreement-dual-signature --output evidence/incidents/$INCIDENT_ID.json`.

### Diagnostic Decision Tree
```text
Engagement Agreement Dual Signature incident decision tree
1. Is EngagementAgreementDualSignatureCritical firing in more than one cell?
   |-- yes: declare fleet incident, page PagerDuty oya-workplace-integration-primary, and run cross-cell containment.
   |-- no: keep scope to the affected cell and continue tenant isolation checks.
2. Does oya_workplace_integration_engagement_agreement_dual_signature_queue_depth grow while oya_workplace_integration_engagement_agreement_dual_signature_error_ratio is flat?
   |-- yes: downstream dependency, replay backlog, or queue-drain issue; choose mitigation branch B.
   |-- no: local regression, bad input, or policy/config drift; continue branch selection.
3. Does audit-chain show EVT_WORKPLACE_INTEGRATION_ENGAGEMENT_AGREEMENT_DUAL_SIGNATURE_INCIDENT gaps?
   |-- yes: do not close; run evidence replay before resolution.
   |-- no: mitigation can proceed after state is green.
4. Is customer, finance, security, or regulator impact confirmed?
   |-- yes: promote severity, open #inc-workplace-integration, and notify compliance or security handoff.
   |-- no: keep internal incident and collect evidence.
```
- Branch A (confirmed WorkplaceAgreement correctness risk): use the matching mitigation block below and record `decision_branch=A` in `EVT_WORKPLACE_INTEGRATION_ENGAGEMENT_AGREEMENT_DUAL_SIGNATURE_INCIDENT`.
- Branch B (dependency saturation or replay backlog): use the matching mitigation block below and record `decision_branch=B` in `EVT_WORKPLACE_INTEGRATION_ENGAGEMENT_AGREEMENT_DUAL_SIGNATURE_INCIDENT`.
- Branch C (policy, permit, or tenant-scope drift): use the matching mitigation block below and record `decision_branch=C` in `EVT_WORKPLACE_INTEGRATION_ENGAGEMENT_AGREEMENT_DUAL_SIGNATURE_INCIDENT`.
- Branch D (customer-visible or regulated evidence gap): use the matching mitigation block below and record `decision_branch=D` in `EVT_WORKPLACE_INTEGRATION_ENGAGEMENT_AGREEMENT_DUAL_SIGNATURE_INCIDENT`.

## Mitigation Steps
1. Acknowledge page: `pd incident ack --service workplace-integration --incident $INCIDENT_ID`.
2. Create bridge: `oya incident bridge create --incident $INCIDENT_ID --channel #inc-workplace-integration --severity sev1`.
3. Freeze risky automation: `oya flags set oya.workplace-integration.engagement_agreement_dual_signature.incident_hold=true --cell $CELL --tenant $TENANT --reason $INCIDENT_ID`.
4. Enable circuit breaker: `oya ops breaker open workplace-integration-engagement-agreement-dual-signature-circuit-breaker --cell $CELL --tenant $TENANT --ttl 30m --reason $INCIDENT_ID`.
5. Reduce blast radius: `kubectl -n workplace-integration scale deploy/workplace-integration-engagement-agreement-dual-signature-worker --replicas=1`.
6. Protect tenant boundary: `oya tenancy quarantine --tenant $TENANT --reason workplace-integration-engagement-agreement-dual-signature --ttl 60m`.
7. Pause promotion: incident hold PR against `dev` (plain `git`; branch-protected `oya-ci-required` cloud-ci/oya-ci acceptance required; local command output is transition evidence only).
8. Drain queue safely: `oya ops workplace-integration engagement-agreement-dual-signature drain --cell $CELL --tenant $TENANT --max-items 500 --dry-run`.
9. Execute bounded drain: `oya ops workplace-integration engagement-agreement-dual-signature drain --cell $CELL --tenant $TENANT --max-items 500 --confirm $INCIDENT_ID`.
10. Replay missing audit events: `oya audit-chain replay --event-class EVT_WORKPLACE_INTEGRATION_ENGAGEMENT_AGREEMENT_DUAL_SIGNATURE_INCIDENT --incident $INCIDENT_ID --from evidence/incidents/$INCIDENT_ID.json`.
11. Rollback last deploy if causal: `kubectl -n workplace-integration rollout undo deploy/workplace-integration-engagement-agreement-dual-signature-worker`.
12. Raise HPA cap if saturation is proven: `kubectl -n workplace-integration patch hpa workplace-integration-engagement-agreement-dual-signature-worker --type merge -p '{"spec":{"maxReplicas":12}}'`.
13. Throttle hot tenant: `oya ops rate-limit set --tenant $TENANT --surface workplace-integration.engagement-agreement-dual-signature --rps 25 --ttl 30m`.
14. Block abusive principal when relevant: `oya identity principal suspend --principal suspected-abuse --tenant $TENANT --reason $INCIDENT_ID`.
15. Protect evidence: `oya evidence freeze --incident $INCIDENT_ID --paths app/workplace-integration/runbooks/engagement-agreement-dual-signature.md,evidence/incidents/$INCIDENT_ID.json`.
16. Notify service owners: `oya notify service-owner --microservice workplace-integration --incident $INCIDENT_ID --channel #inc-workplace-integration`.
17. Open external vendor ticket: `oya vendor ticket open --vendor "DocuSign enterprise support" --incident $INCIDENT_ID --summary workplace-integration-engagement-agreement-dual-signature`.
18. Confirm breaker effect: `oya ops breaker status workplace-integration-engagement-agreement-dual-signature-circuit-breaker --cell $CELL --tenant $TENANT --expect open`.
19. Confirm user impact reduced: `curl -s https://workplace-integration.internal.oyatie.dev/v1/workplace-integration/engagement-agreement-dual-signature/incident-handoff/health -H "x-oya-tenant: $TENANT"`.
20. Emit mitigation audit: `oya audit-chain emit --event-class EVT_WORKPLACE_INTEGRATION_ENGAGEMENT_AGREEMENT_DUAL_SIGNATURE_INCIDENT --incident $INCIDENT_ID --field mitigation=active --field runbook=engagement-agreement-dual-signature`.

### Mitigation Branch Guidance
- Branch A: confirmed WorkplaceAgreement correctness risk.
  - Required action: keep `workplace-integration-engagement-agreement-dual-signature-circuit-breaker` open until `oya_workplace_integration_engagement_agreement_dual_signature_error_ratio` is below 0.005 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/workplace-integration-ops/engagement-agreement-dual-signature?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=110` to the incident.
  - Required audit: emit `EVT_WORKPLACE_INTEGRATION_ENGAGEMENT_AGREEMENT_DUAL_SIGNATURE_INCIDENT` with `branch=A`, `operator_id`, and `evidence_hash`.
- Branch B: dependency saturation or replay backlog.
  - Required action: keep `workplace-integration-engagement-agreement-dual-signature-circuit-breaker` open until `oya_workplace_integration_engagement_agreement_dual_signature_error_ratio` is below 0.01 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/workplace-integration-ops/engagement-agreement-dual-signature?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=111` to the incident.
  - Required audit: emit `EVT_WORKPLACE_INTEGRATION_ENGAGEMENT_AGREEMENT_DUAL_SIGNATURE_INCIDENT` with `branch=B`, `operator_id`, and `evidence_hash`.
- Branch C: policy, permit, or tenant-scope drift.
  - Required action: keep `workplace-integration-engagement-agreement-dual-signature-circuit-breaker` open until `oya_workplace_integration_engagement_agreement_dual_signature_error_ratio` is below 0.005 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/workplace-integration-ops/engagement-agreement-dual-signature?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=112` to the incident.
  - Required audit: emit `EVT_WORKPLACE_INTEGRATION_ENGAGEMENT_AGREEMENT_DUAL_SIGNATURE_INCIDENT` with `branch=C`, `operator_id`, and `evidence_hash`.
- Branch D: customer-visible or regulated evidence gap.
  - Required action: keep `workplace-integration-engagement-agreement-dual-signature-circuit-breaker` open until `oya_workplace_integration_engagement_agreement_dual_signature_error_ratio` is below 0.01 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/workplace-integration-ops/engagement-agreement-dual-signature?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=113` to the incident.
  - Required audit: emit `EVT_WORKPLACE_INTEGRATION_ENGAGEMENT_AGREEMENT_DUAL_SIGNATURE_INCIDENT` with `branch=D`, `operator_id`, and `evidence_hash`.

## Resolution Steps
1. Identify code owner path: `rg "engagement_agreement_dual_signature|EngagementAgreementDualSignatureCritical|workplace_integration.engagement_agreement_dual_signature.incident_state" crates app/workplace-integration -g "!app/workplace-integration/runbooks/**"`.
2. Patch domain invariant: `edit WorkplaceAgreement domain where engagement_agreement_dual_signature state transition is validated`.
3. Patch API guard: `edit app/workplace-integration/contracts/openapi-v1.yaml if the failing path is north-south or async handoff`.
4. Patch policy: `edit app/workplace-integration/policies/esign-initiate.cedar with explicit deny/permit branch and tenant/cell scope`.
5. Patch runtime config: `edit app/workplace-integration/iac/kustomize/base/kustomization.yaml if deploy/config drift caused the incident`.
6. Add regression test: `cargo test -p WorkplaceAgreement domain engagement_agreement_dual_signature_incident_regression -- --nocapture`.
7. Add gate evidence from the `oya-ci-required` workplace-integration `engagement-agreement-dual-signature` fixture lane for `incident-engagement-agreement-dual-signature.json`.
8. Add SLO assertion: `update app/workplace-integration/slos/clock-attestation-availability.openslo.yaml with alert EngagementAgreementDualSignatureCritical when this was a missing alert`.
9. Add dashboard panel: `update app/workplace-integration/dashboards/audit-evidence.json with oya_workplace_integration_engagement_agreement_dual_signature_error_ratio, oya_workplace_integration_engagement_agreement_dual_signature_lag_seconds, and oya_workplace_integration_dual_signature_stall_total`.
10. Rebuild affected crate: `cargo check -p WorkplaceAgreement domain --all-targets`.
11. Run targeted tests: `cargo test -p WorkplaceAgreement domain --all-features`.
12. Run the `oya-ci-required` workplace-integration policy validation lane for the `workplace-integration` microservice.
13. Deploy canary: `oya deploy canary --microservice workplace-integration --component workplace-integration-engagement-agreement-dual-signature-worker --cell $CELL --weight 1`.
14. Watch burn rate: `oya ops watch --metric oya_workplace_integration_engagement_agreement_dual_signature_error_ratio --threshold 0.005 --window 30m --cell $CELL`.
15. Close circuit breaker: `oya ops breaker close workplace-integration-engagement-agreement-dual-signature-circuit-breaker --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
16. Unfreeze automation: `oya flags set oya.workplace-integration.engagement_agreement_dual_signature.incident_hold=false --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
17. Resume promotion: recovery PR against `dev` (plain `git`; branch-protected `oya-ci-required` cloud-ci/oya-ci acceptance required; local command output is transition evidence only).
18. Seal resolution audit: `oya audit-chain emit --event-class EVT_WORKPLACE_INTEGRATION_ENGAGEMENT_AGREEMENT_DUAL_SIGNATURE_INCIDENT --incident $INCIDENT_ID --field resolution=complete --field runbook=engagement-agreement-dual-signature`.
19. Verify seal: `oya audit-chain verify --event-class EVT_WORKPLACE_INTEGRATION_ENGAGEMENT_AGREEMENT_DUAL_SIGNATURE_INCIDENT --incident $INCIDENT_ID`.
20. Attach final evidence: `oya evidence attach --incident $INCIDENT_ID --file evidence/incidents/$INCIDENT_ID.json --kind final-resolution`.

### Code Paths To Inspect First
- `WorkplaceAgreement domain`: inspect for `engagement_agreement_dual_signature` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `ESignSession usecase`: inspect for `engagement_agreement_dual_signature` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `roster-binding worker`: inspect for `engagement_agreement_dual_signature` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `clock-attestation adapter`: inspect for `engagement_agreement_dual_signature` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `app/workplace-integration/contracts/openapi-v1.yaml`: verify request/response or event contract only when incident evidence points there.
- `app/workplace-integration/contracts/asyncapi-v1.yaml`: verify request/response or event contract only when incident evidence points there.
- `app/workplace-integration/contracts/workplace-integration-v1.proto`: verify request/response or event contract only when incident evidence points there.
- `app/workplace-integration/dashboards/audit-evidence.json`: verify panel coverage for `oya_workplace_integration_engagement_agreement_dual_signature_error_ratio`, `oya_workplace_integration_engagement_agreement_dual_signature_lag_seconds`, and `oya_workplace_integration_dual_signature_stall_total`.
- `app/workplace-integration/slos/`: verify alert vocabulary and threshold alignment before changing runtime thresholds.
- `app/workplace-integration/policies/`: verify policy branch ownership before relaxing deny rules or emergency bypasses.

## Verification Checklist
- `EngagementAgreementDualSignatureCritical` and `EngagementAgreementDualSignatureSloBurn` are both resolved in Alertmanager for 30 minutes.
- `oya_workplace_integration_engagement_agreement_dual_signature_error_ratio < 0.005` for 3 consecutive 10 minute windows.
- `oya_workplace_integration_engagement_agreement_dual_signature_lag_seconds < 120` for all production cells.
- `oya_workplace_integration_engagement_agreement_dual_signature_queue_depth` is draining and not growing for the affected tenant.
- Service-specific signal `oya_workplace_integration_dual_signature_stall_total` is below the threshold documented in `app/workplace-integration/slos/clock-attestation-availability.openslo.yaml`.
- Dashboard `https://grafana.dev.oyatie.internal/d/workplace-integration-ops/engagement-agreement-dual-signature?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=101` shows green panels for the affected cell.
- Audit-chain query for `EVT_WORKPLACE_INTEGRATION_ENGAGEMENT_AGREEMENT_DUAL_SIGNATURE_INCIDENT` returns mitigation and resolution events.
- Circuit breaker `workplace-integration-engagement-agreement-dual-signature-circuit-breaker` is closed after rollback window.
- Feature flag `oya.workplace-integration.engagement_agreement_dual_signature.incident_hold` is false for the affected tenant unless long-term hold is approved.
- Runbook invocation evidence is attached to `evidence/incidents/$INCIDENT_ID.json`.
- Service owner acknowledged final handoff in `#inc-workplace-integration`.

## Capacity and Rollback Guardrails
- Capacity math: if `oya_workplace_integration_engagement_agreement_dual_signature_queue_depth` is 5000 and the worker drains 25 items/second, the best-case drain is 200 seconds before retries; page earlier when drain time exceeds 300 seconds.
- Capacity math: with 12 replicas at 25 items/second each, the hard ceiling is 300 items/second; keep tenant throttle below 25 RPS until error ratio stays below 0.005.
- Rollback checkpoint 1: before changing `oya.workplace-integration.engagement_agreement_dual_signature.incident_hold`, snapshot current value with `oya flags get oya.workplace-integration.engagement_agreement_dual_signature.incident_hold --output json`.
- Rollback checkpoint 2: before opening `workplace-integration-engagement-agreement-dual-signature-circuit-breaker`, capture `oya_workplace_integration_engagement_agreement_dual_signature_request_rate` and `oya_workplace_integration_engagement_agreement_dual_signature_success_ratio` from Mimir.
- Rollback checkpoint 3: before scaling deployments, capture `kubectl -n workplace-integration get deploy workplace-integration-engagement-agreement-dual-signature-worker -o yaml`.
- Rollback command for flag: `oya flags set oya.workplace-integration.engagement_agreement_dual_signature.incident_hold=false --cell $CELL --tenant $TENANT --reason rollback-$INCIDENT_ID`.
- Rollback command for breaker: `oya ops breaker close workplace-integration-engagement-agreement-dual-signature-circuit-breaker --cell $CELL --tenant $TENANT --reason rollback-$INCIDENT_ID`.
- Rollback command for deployment: `kubectl -n workplace-integration rollout undo deploy/workplace-integration-engagement-agreement-dual-signature-worker`.
- Rollback command for tenant throttle: `oya ops rate-limit clear --tenant $TENANT --surface workplace-integration.engagement-agreement-dual-signature --reason rollback-$INCIDENT_ID`.
- Stop rollback if `EVT_WORKPLACE_INTEGRATION_ENGAGEMENT_AGREEMENT_DUAL_SIGNATURE_INCIDENT` cannot be emitted; preserve the current state and escalate to audit-chain before additional mutation.

## Postmortem Template
Use this exact skeleton for the incident document. The field names are intentionally stable for ADR-0263 audit emission extraction.
```markdown
---
doc_class: IncidentPostmortem
runbook_id: workplace-integration-engagement-agreement-dual-signature
microservice: workplace-integration
event_class: EVT_WORKPLACE_INTEGRATION_ENGAGEMENT_AGREEMENT_DUAL_SIGNATURE_INCIDENT
incident_id: <INC-...>
severity: sev1
status: draft
detected_at: <UTC>
mitigated_at: <UTC>
resolved_at: <UTC>
commander: <handle>
evidence_hash: <sha256>
---

# Engagement Agreement Dual Signature postmortem

## Summary
- What happened in workplace-integration/engagement-agreement-dual-signature.
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
- Emit EVT_WORKPLACE_INTEGRATION_ENGAGEMENT_AGREEMENT_DUAL_SIGNATURE_INCIDENT with incident_id, tenant_id, cell_id, principal_id, decision_id, evidence_hash, operator_id, runbook_id.
- Attach dashboard snapshot URLs and command transcripts.
- Seal mitigation and resolution events before closure.

## Corrective Actions
- Action, owner, due date, validation command, linked issue.
```

## Escalation Path
- Primary on-call: PagerDuty oya-workplace-integration-primary; hris-esign-secondary.
- Incident SLA: ack 3m for sev0/sev1, 10m for sev2, 30m for sev3; status update every 10m until the critical alert clears.
- Incident commander: first responder from axis-workplace-integration + ops-sre-reliability; transfer only by explicit message in `#inc-workplace-integration`.
- Security escalation: page `ops-security-primary` immediately for sev0, credential, cross-tenant, fraud, or audit-seal symptoms.
- Compliance escalation: page `dpo-office-duty` when tenant data, regulator evidence, money movement, or breach-clock symptoms are present.
- Architecture escalation: page `architecture-reviewer` before manual bypass, policy rollback, or invariant relaxation.
- External vendors: DocuSign enterprise support; Workday HCM support; ADP Workforce Now support. Open a ticket once local dependency health is proven and vendor dependency remains suspect.
- Customer communications: use status page component `oyatie-workplace-integration-engagement-agreement-dual-signature` and keep private details in the incident channel.
- Regulatory clock: if tenant data, financial correctness, or evidence integrity is possibly affected, start the compliance 72h assessment timer even if exposure is unconfirmed.
- Executive notice: sev0 or fleet-wide sev1 goes to `#exec-incident-readout` within 30 minutes.

## Cross-µservice Coordination
- Notify `identity`: `oya incident handoff --target identity --source workplace-integration --runbook engagement-agreement-dual-signature --incident $INCIDENT_ID --severity sev1 --branch A`; expect `202 accepted`.
- Require `identity` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `mail`: `oya incident handoff --target mail --source workplace-integration --runbook engagement-agreement-dual-signature --incident $INCIDENT_ID --severity sev1 --branch B`; expect `202 accepted`.
- Require `mail` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `drive`: `oya incident handoff --target drive --source workplace-integration --runbook engagement-agreement-dual-signature --incident $INCIDENT_ID --severity sev1 --branch C`; expect `202 accepted`.
- Require `drive` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `workflow-engine`: `oya incident handoff --target workflow-engine --source workplace-integration --runbook engagement-agreement-dual-signature --incident $INCIDENT_ID --severity sev1 --branch D`; expect `202 accepted`.
- Require `workflow-engine` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `community`: `oya incident handoff --target community --source workplace-integration --runbook engagement-agreement-dual-signature --incident $INCIDENT_ID --severity sev1 --branch A`; expect `202 accepted`.
- Require `community` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Observability handoff API: `oya incident handoff --target observability --source workplace-integration --runbook engagement-agreement-dual-signature --incident $INCIDENT_ID`.
- Governance handoff API: `oya incident handoff --target governance --source workplace-integration --runbook engagement-agreement-dual-signature --incident $INCIDENT_ID`.
- Compliance handoff API: `oya incident handoff --target compliance --source workplace-integration --runbook engagement-agreement-dual-signature --incident $INCIDENT_ID`.
- Audit-chain handoff API: `oya incident handoff --target audit-chain --source workplace-integration --runbook engagement-agreement-dual-signature --incident $INCIDENT_ID`.
- Tenancy handoff API: `oya incident handoff --target tenancy --source workplace-integration --runbook engagement-agreement-dual-signature --incident $INCIDENT_ID`.

## Handoff Notes
- Do not hand off with only the alert name; include `oya_workplace_integration_engagement_agreement_dual_signature_error_ratio`, `oya_workplace_integration_engagement_agreement_dual_signature_lag_seconds`, `oya_workplace_integration_engagement_agreement_dual_signature_queue_depth`, `oya_workplace_integration_dual_signature_stall_total`, current breaker state, and audit seal status.
- Keep `workplace-integration-engagement-agreement-dual-signature-circuit-breaker` owner as axis-workplace-integration + ops-sre-reliability until the receiving service explicitly accepts.
- If another runbook owns the downstream fix, link this incident as upstream and keep this runbook open until downstream verification returns green.
- Close only after `EVT_WORKPLACE_INTEGRATION_ENGAGEMENT_AGREEMENT_DUAL_SIGNATURE_INCIDENT` has a sealed resolution row and every coordination endpoint above has either accepted or explicitly declined scope.

## Sources Checked During This Substance Pass
- `app/workplace-integration/dashboards/` for dashboard names and operational panels: audit-evidence.json, policy-deny-rate.json, replay-health.json, service-overview.json, tenant-slo-burn.json.
- `app/workplace-integration/slos/` for OpenSLO alert vocabulary and threshold alignment: clock-attestation-availability.openslo.yaml, dlp-trace-seal-fidelity.openslo.yaml, esign-initiate-availability.openslo.yaml, offer-generation-latency.openslo.yaml, roster-binding-accuracy.openslo.yaml, signature-capture-latency.openslo.yaml.
- `app/workplace-integration/policies/` for named policy and authorization surfaces: clock-attest.cedar, dlp-trace-seal.cedar, esign-initiate.cedar, esign-sign.cedar, offer-generate.cedar, roster-bind.cedar.
- `app/workplace-integration/contracts/` for API, AsyncAPI, proto, and adapter surfaces: contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/workplace-integration-v1.proto.
- `app/workplace-integration/manifest.json` for owner, dependency, capability, and bounded-context vocabulary; topic `engagement-agreement-dual-signature` is the scenario anchor.

## Checkpoint Closure Criteria
- The runbook remains current when `EngagementAgreementDualSignatureCritical`, `EngagementAgreementDualSignatureSloBurn`, `oya_workplace_integration_dual_signature_stall_total`, `oya.workplace-integration.engagement_agreement_dual_signature.incident_hold`, and `workplace-integration-engagement-agreement-dual-signature-circuit-breaker` all resolve to live telemetry, flag, or breaker records.
- The incident is cleanly halted if required authority is missing for tenant quarantine, policy rollback, or vendor escalation; do not improvise outside the named commands.
- The checkpoint is complete when the branch-protected `oya-ci-required` cloud-ci/oya-ci gate accepts the runbook evidence for the five target scopes; local command output is transition evidence only, not destination authority.
