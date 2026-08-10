---
doc_class: Runbook
title: PHI Access Anomaly
status: Accepted
date: 2026-05-20
microservice: compliance
severity: sev0
audience: compliance-officer
owner_team: axis-compliance + DPO office + ops-security
source_wave: codex-runbooks-substrate-w1
change_scope: substance rewrite of thin existing runbook
doc_status: published
---

# Runbook: PHI Access Anomaly

## Operator Contract
- Runbook id: compliance-phi-access-anomaly.
- Primary service namespace: `compliance`.
- Owning rotation: PagerDuty oya-compliance-primary; DPO escalation bridge; legal-duty officer.
- Incident channel: `#inc-compliance-regulatory`.
- External dependencies: Sigstore support; SeaweedFS support; external auditor portal support; regulator submission portal desk.
- API authority: `https://compliance.internal.oyatie.dev/v1/compliance/phi-access-anomaly/incident-handoff`.
- Audit event class: `EVT-COMPLIANCE-PHI_ACCESS_ANOMALY-INCIDENT` with ADR-0263 fields `incident_id`, `tenant_id`, `cell_id`, `microservice`, `runbook_id`, `decision_id`, `evidence_hash`, `operator_id`.
- Stop condition: mitigation has held for 30 minutes, CompliancePhiAccessAnomalyCritical is green, and all handoff APIs in Cross-µservice Coordination return `202 accepted`.
- Safety invariant: never clear the incident until `EVT-COMPLIANCE-PHI_ACCESS_ANOMALY-INCIDENT` is sealed and the postmortem skeleton exists under `evidence/postmortems/compliance-phi-access-anomaly-<incident-id>.md`.

## Trigger Conditions
- Page on alert `CompliancePhiAccessAnomalyCritical` when `oya_compliance_phi_access_anomaly_error_ratio > 0.02` for 10 minutes in any production cell.
- Page on alert `CompliancePhiAccessAnomalySloBurn` when `oya_compliance_phi_access_anomaly_lag_seconds > 300` for 2 consecutive evaluator windows.
- Open a sev0 if `oya_compliance_phi_access_anomaly_correctness_ratio < 0.9999` and the affected label set includes `tenant_id` or `principal_id`.
- Open a sev1 if `oya_compliance_phi_access_anomaly_queue_depth > 5000` for 15 minutes or retry backlog grows by more than 20 percent in one 5 minute window.
- Trigger from customer report when Support tags the case `compliance.phi-access-anomaly.customer_visible` in Zendesk.
- Trigger from CI when `cargo run -p oya-dev-cli -- gate validate compliance-phi-access-anomaly --production-snapshot` exits non-zero against the latest production evidence bundle.
- Primary dashboard: `https://grafana.dev.oyatie.internal/d/compliance-substrate/phi-access-anomaly?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=102`.
- Secondary dashboard: `https://grafana.dev.oyatie.internal/d/compliance-substrate/phi-access-anomaly?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=210`.
- Loki explorer: `https://grafana.dev.oyatie.internal/explore?query={namespace="compliance",runbook="phi-access-anomaly"}`.
- Alertmanager route: `oyatie-compliance-phi-access-anomaly-critical`; silence only with incident commander approval and `EVT-COMPLIANCE-PHI_ACCESS_ANOMALY-INCIDENT` evidence.
- Synthetic probe: `oya ops probe compliance phi-access-anomaly --cell prod-us-east-1 --tenant synthetic-canary` returns `healthy=true`.
- Drift detector: `registry/compliance/phi-access-anomaly/expected-state.json` hash differs from live `https://compliance.internal.oyatie.dev/v1/admin/state-hash`.

## Symptoms
- User-facing impact: phi access anomaly blocks or corrupts the compliance control path for affected tenants.
- Operators see Grafana panel `audit-chain-seal-health / PHI Access Anomaly burn rate` turn red before the primary alert resolves.
- Loki signature `compliance.phi_access_anomaly.incident_state=failed` appears with fields `incident_id`, `tenant_id`, `cell_id`, `decision_id`, `evidence_hash`.
- Kubernetes events include `reason=CompliancePhiAccessAnomalyDegraded` on deployment `compliance-phi-access-anomaly-worker`.
- Audit-chain shows missing or delayed `EVT-COMPLIANCE-PHI_ACCESS_ANOMALY-INCIDENT` entries when queried with `oya audit-chain query --event-class EVT-COMPLIANCE-PHI_ACCESS_ANOMALY-INCIDENT --since 30m`.
- Metric pattern: `oya_compliance_phi_access_anomaly_error_ratio` rises before `oya_compliance_phi_access_anomaly_lag_seconds`; if lag rises first, suspect dependency saturation rather than local regression.
- Metric pattern: `oya_compliance_phi_access_anomaly_queue_depth` increases while pod CPU stays below 40 percent; suspect downstream refusal or feature flag deadlock.
- Tenant-specific shape: one `tenant_id` dominates labels in `oya_compliance_phi_access_anomaly_queue_depth`; isolate before fleet mitigation.
- Fleet-wide shape: at least three cells report `CompliancePhiAccessAnomalyCritical` in one 15 minute window; switch to sev1 bridge even if individual tenants are low-volume.
- Log signature `decision=deny reason=phi-access-anomaly.policy_guard` means the guard is working; investigate caller inputs before rollback.
- Log signature `decision=permit reason=phi-access-anomaly.break_glass` means manual intervention is active; confirm two-person authorization.
- Log signature `audit_emit_status=stalled event_class=EVT-COMPLIANCE-PHI_ACCESS_ANOMALY-INCIDENT` means mitigation cannot be closed until replay succeeds.

## Diagnostic Steps
1. Set incident variables: `export INCIDENT_ID=INC-compliance-phi-access-anomaly-$(date -u +%Y%m%dT%H%M%SZ); export CELL=prod-us-east-1; export TENANT=synthetic-canary`.
2. Confirm active alerts: `curl -s https://compliance.internal.oyatie.dev/v1/alerts?runbook=phi-access-anomaly | jq .alerts`.
3. Check Kubernetes rollout: `kubectl -n compliance rollout status deploy/compliance-phi-access-anomaly-worker --timeout=60s`.
4. List unhealthy pods: `kubectl -n compliance get pods -l app=compliance-phi-access-anomaly -o wide`.
5. Read structured logs: `kubectl -n compliance logs deploy/compliance-phi-access-anomaly-worker --since=30m | rg "compliance.phi_access_anomaly.incident_state|CompliancePhiAccessAnomalyCritical|EVT-COMPLIANCE-PHI_ACCESS_ANOMALY-INCIDENT"`.
6. Query Loki directly: `logcli query '{namespace="compliance",runbook="phi-access-anomaly"}' --since=30m --limit=200`.
7. Check Prometheus fast burn: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_compliance_phi_access_anomaly_error_ratio{cell="prod-us-east-1"}'`.
8. Check lag: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_compliance_phi_access_anomaly_lag_seconds{cell="prod-us-east-1"}'`.
9. Check queue: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_compliance_phi_access_anomaly_queue_depth{cell="prod-us-east-1"}'`.
10. Open primary dashboard: `open "https://grafana.dev.oyatie.internal/d/compliance-substrate/phi-access-anomaly?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=102&var-incident=$INCIDENT_ID"`.
11. Open secondary dashboard: `open "https://grafana.dev.oyatie.internal/d/compliance-substrate/phi-access-anomaly?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=210&var-tenant=$TENANT"`.
12. Verify audit-chain emission: `oya audit-chain query --event-class EVT-COMPLIANCE-PHI_ACCESS_ANOMALY-INCIDENT --since 30m --cell $CELL --tenant $TENANT`.
13. Verify service state: `oya ops compliance phi-access-anomaly status --cell $CELL --tenant $TENANT --output json`.
14. Run production snapshot gate: `cargo run -p oya-dev-cli -- gate validate compliance-phi-access-anomaly --production-snapshot --cell $CELL`.
15. Check Cargo owner crate: `cargo test -p oya-compliance-domain phi_access_anomaly -- --nocapture`.
16. Check API contract smoke: `curl -s https://compliance.internal.oyatie.dev/v1/compliance/phi-access-anomaly/incident-handoff -H "x-oya-tenant: $TENANT"`.
17. Inspect config: `kubectl -n compliance get configmap compliance-phi-access-anomaly-config -o yaml`.
18. Inspect feature flags: `oya flags get oya.compliance.phi_access_anomaly.incident_hold --cell $CELL --tenant $TENANT --output yaml`.
19. Inspect circuit breaker: `oya ops breaker status compliance-phi-access-anomaly-circuit-breaker --cell $CELL --tenant $TENANT`.
20. Check recent deploy: `kubectl -n compliance rollout history deploy/compliance-phi-access-anomaly-worker | tail -20`.
21. Check policy file: `test -f compliance/policy/pack-overlay-authorization.cedar`.
22. Check SLO files: `ls compliance/observability/slos/*.openslo.yaml | sort`.
23. Check catalog components: `find compliance/catalog -maxdepth 1 -type f | sort | rg "compliance|phi"`.
24. Confirm no cross-cell spread: `oya ops cells query --metric oya_compliance_phi_access_anomaly_error_ratio --window 30m --threshold 0.02`.
25. Snapshot evidence: `oya evidence snapshot --incident $INCIDENT_ID --microservice compliance --runbook phi-access-anomaly --output evidence/incidents/$INCIDENT_ID.json`.

### Diagnostic Decision Tree
```text
PHI Access Anomaly incident decision tree
1. Is CompliancePhiAccessAnomalyCritical firing in more than one cell?
   |-- yes: declare fleet incident, page PagerDuty oya-compliance-primary; DPO escalation bridge; legal-duty officer, and run cross-cell containment.
   |-- no: keep scope to the affected cell and continue tenant isolation checks.
2. Does oya_compliance_phi_access_anomaly_queue_depth grow while oya_compliance_phi_access_anomaly_error_ratio is flat?
   |-- yes: downstream dependency or replay backlog; choose mitigation branch B.
   |-- no: local regression or bad input; continue branch selection.
3. Does audit-chain show EVT-COMPLIANCE-PHI_ACCESS_ANOMALY-INCIDENT gaps?
   |-- yes: do not close; run evidence replay before resolution.
   |-- no: mitigation can proceed after state is green.
4. Is customer or regulator impact confirmed?
   |-- yes: promote severity, open #inc-compliance-regulatory, and notify compliance handoff.
   |-- no: keep internal incident and collect evidence.
```
- Branch A (confirmed security boundary failure): use the matching mitigation block below and record `decision_branch=A` in `EVT-COMPLIANCE-PHI_ACCESS_ANOMALY-INCIDENT`.
- Branch B (suspected false positive): use the matching mitigation block below and record `decision_branch=B` in `EVT-COMPLIANCE-PHI_ACCESS_ANOMALY-INCIDENT`.
- Branch C (forensic evidence unavailable): use the matching mitigation block below and record `decision_branch=C` in `EVT-COMPLIANCE-PHI_ACCESS_ANOMALY-INCIDENT`.
- Branch D (customer or regulator visible impact): use the matching mitigation block below and record `decision_branch=D` in `EVT-COMPLIANCE-PHI_ACCESS_ANOMALY-INCIDENT`.

## Mitigation Steps
1. Acknowledge page: `pd incident ack --service compliance --incident $INCIDENT_ID`.
2. Create bridge: `oya incident bridge create --incident $INCIDENT_ID --channel #inc-compliance-regulatory --severity sev0`.
3. Freeze risky automation: `oya flags set oya.compliance.phi_access_anomaly.incident_hold=true --cell $CELL --tenant $TENANT --reason $INCIDENT_ID`.
4. Enable circuit breaker: `oya ops breaker open compliance-phi-access-anomaly-circuit-breaker --cell $CELL --tenant $TENANT --ttl 30m --reason $INCIDENT_ID`.
5. Reduce blast radius: `kubectl -n compliance scale deploy/compliance-phi-access-anomaly-worker --replicas=1`.
6. Protect tenant boundary: `oya tenancy quarantine --tenant $TENANT --reason compliance-phi-access-anomaly --ttl 60m`.
7. Pause promotion: incident hold PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
8. Drain queue safely: `oya ops compliance phi-access-anomaly drain --cell $CELL --tenant $TENANT --max-items 500 --dry-run`.
9. Execute bounded drain: `oya ops compliance phi-access-anomaly drain --cell $CELL --tenant $TENANT --max-items 500 --confirm $INCIDENT_ID`.
10. Replay missing audit events: `oya audit-chain replay --event-class EVT-COMPLIANCE-PHI_ACCESS_ANOMALY-INCIDENT --incident $INCIDENT_ID --from evidence/incidents/$INCIDENT_ID.json`.
11. Rollback last deploy if causal: `kubectl -n compliance rollout undo deploy/compliance-phi-access-anomaly-worker`.
12. Raise HPA cap if saturation: `kubectl -n compliance patch hpa compliance-phi-access-anomaly-worker --type merge -p '{"spec":{"maxReplicas":12}}'`.
13. Throttle hot tenant: `oya ops rate-limit set --tenant $TENANT --surface compliance.phi-access-anomaly --rps 25 --ttl 30m`.
14. Block abusive principal: `oya identity principal suspend --principal suspected-abuse --tenant $TENANT --reason $INCIDENT_ID`.
15. Protect evidence: `oya evidence freeze --incident $INCIDENT_ID --paths compliance/runbooks/phi-access-anomaly.md,evidence/incidents/$INCIDENT_ID.json`.
16. Notify service owners: `oya notify service-owner --microservice compliance --incident $INCIDENT_ID --channel #inc-compliance-regulatory`.
17. Open external vendor ticket: `oya vendor ticket open --vendor primary-compliance --incident $INCIDENT_ID --summary phi-access-anomaly`.
18. Confirm breaker effect: `oya ops breaker status compliance-phi-access-anomaly-circuit-breaker --cell $CELL --tenant $TENANT --expect open`.
19. Confirm user impact reduced: `curl -s https://compliance.internal.oyatie.dev/v1/compliance/phi-access-anomaly/incident-handoff/health -H "x-oya-tenant: $TENANT"`.
20. Emit mitigation audit: `oya audit-chain emit --event-class EVT-COMPLIANCE-PHI_ACCESS_ANOMALY-INCIDENT --incident $INCIDENT_ID --field mitigation=active --field runbook=phi-access-anomaly`.

### Mitigation Branch Guidance
- Branch A: confirmed security boundary failure.
  - Required action: keep `compliance-phi-access-anomaly-circuit-breaker` open until `oya_compliance_phi_access_anomaly_error_ratio` is below 0.005 for 3 windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/compliance-substrate/phi-access-anomaly?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=102` to the incident.
  - Required audit: emit `EVT-COMPLIANCE-PHI_ACCESS_ANOMALY-INCIDENT` with `branch=A`, `operator_id`, and `evidence_hash`.
- Branch B: suspected false positive.
  - Required action: keep `compliance-phi-access-anomaly-circuit-breaker` open until `oya_compliance_phi_access_anomaly_error_ratio` is below 0.005 for 3 windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/compliance-substrate/phi-access-anomaly?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=103` to the incident.
  - Required audit: emit `EVT-COMPLIANCE-PHI_ACCESS_ANOMALY-INCIDENT` with `branch=B`, `operator_id`, and `evidence_hash`.
- Branch C: forensic evidence unavailable.
  - Required action: keep `compliance-phi-access-anomaly-circuit-breaker` open until `oya_compliance_phi_access_anomaly_error_ratio` is below 0.005 for 3 windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/compliance-substrate/phi-access-anomaly?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=104` to the incident.
  - Required audit: emit `EVT-COMPLIANCE-PHI_ACCESS_ANOMALY-INCIDENT` with `branch=C`, `operator_id`, and `evidence_hash`.
- Branch D: customer or regulator visible impact.
  - Required action: keep `compliance-phi-access-anomaly-circuit-breaker` open until `oya_compliance_phi_access_anomaly_error_ratio` is below 0.005 for 3 windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/compliance-substrate/phi-access-anomaly?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=105` to the incident.
  - Required audit: emit `EVT-COMPLIANCE-PHI_ACCESS_ANOMALY-INCIDENT` with `branch=D`, `operator_id`, and `evidence_hash`.

## Resolution Steps
1. Identify code owner path: `rg "phi_access_anomaly|CompliancePhiAccessAnomalyCritical|compliance.phi_access_anomaly.incident_state" crates compliance -g "!compliance/runbooks/**"`.
2. Patch domain invariant: `edit oya-compliance-domain where phi_access_anomaly state transition is validated`.
3. Patch API guard: `edit compliance/contracts/openapi.yaml or catalog REST binding if the failing path is north-south`.
4. Patch policy: `edit compliance/policy/pack-overlay-authorization.cedar or .md with explicit deny/permit branch`.
5. Patch runtime config: `edit compliance/iac/k8s-deployment.yaml or secret-bindings.yaml if deploy/config drift caused the incident`.
6. Add regression test: `cargo test -p oya-compliance-domain phi_access_anomaly_incident_regression -- --nocapture`.
7. Add gate evidence: `cargo run -p oya-dev-cli -- gate validate compliance-phi-access-anomaly --fixture incident-phi-access-anomaly.json`.
8. Add SLO assertion: `update compliance/observability/slos/* with alert CompliancePhiAccessAnomalyCritical when this was a missing alert`.
9. Add dashboard panel: `update compliance/dashboards/audit-chain-seal-health.json with oya_compliance_phi_access_anomaly_error_ratio, oya_compliance_phi_access_anomaly_lag_seconds, and oya_compliance_phi_access_anomaly_queue_depth`.
10. Rebuild affected crate: `cargo check -p oya-compliance-domain --all-targets`.
11. Run targeted tests: `cargo test -p oya-compliance-domain --all-features`.
12. Run policy validation: `cargo run -p oya-dev-cli -- gate validate compliance-policy --microservice compliance`.
13. Deploy canary: `oya deploy canary --microservice compliance --component phi-access-anomaly-worker --cell $CELL --weight 1`.
14. Watch burn rate: `oya ops watch --metric oya_compliance_phi_access_anomaly_error_ratio --threshold 0.005 --window 30m --cell $CELL`.
15. Close circuit breaker: `oya ops breaker close compliance-phi-access-anomaly-circuit-breaker --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
16. Unfreeze automation: `oya flags set oya.compliance.phi_access_anomaly.incident_hold=false --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
17. Resume promotion: recovery PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
18. Seal resolution audit: `oya audit-chain emit --event-class EVT-COMPLIANCE-PHI_ACCESS_ANOMALY-INCIDENT --incident $INCIDENT_ID --field resolution=complete --field runbook=phi-access-anomaly`.
19. Verify seal: `oya audit-chain verify --event-class EVT-COMPLIANCE-PHI_ACCESS_ANOMALY-INCIDENT --incident $INCIDENT_ID`.
20. Attach final evidence: `oya evidence attach --incident $INCIDENT_ID --file evidence/incidents/$INCIDENT_ID.json --kind final-resolution`.

### Code Paths To Inspect First
- `oya-compliance-domain`: inspect for phi_access_anomaly invariants, alert emission, and ADR-0263 evidence fields before touching adjacent code path 1.
- `oya-regional-pack-domain`: inspect for phi_access_anomaly invariants, alert emission, and ADR-0263 evidence fields before touching adjacent code path 2.
- `oya-regional-pack-api`: inspect for phi_access_anomaly invariants, alert emission, and ADR-0263 evidence fields before touching adjacent code path 3.
- `compliance/contracts/`: verify this surface only when the incident evidence points there.
- `compliance/dashboards/audit-chain-seal-health.json`: verify this surface only when the incident evidence points there.
- `compliance/observability/slos/`: verify this surface only when the incident evidence points there.
- `compliance/policy/pack-overlay-authorization.*`: verify this surface only when the incident evidence points there.

## Verification Checklist
- CompliancePhiAccessAnomalyCritical and CompliancePhiAccessAnomalySloBurn are both resolved in Alertmanager for 30 minutes.
- oya_compliance_phi_access_anomaly_error_ratio < 0.005 for 3 consecutive 10 minute windows.
- oya_compliance_phi_access_anomaly_lag_seconds < 120 for all production cells.
- oya_compliance_phi_access_anomaly_queue_depth is draining and not growing for the affected tenant.
- dashboard https://grafana.dev.oyatie.internal/d/compliance-substrate/phi-access-anomaly?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=102 shows green panels for the affected cell.
- audit-chain query for EVT-COMPLIANCE-PHI_ACCESS_ANOMALY-INCIDENT returns mitigation and resolution events.
- circuit breaker compliance-phi-access-anomaly-circuit-breaker is closed after rollback window.
- feature flag oya.compliance.phi_access_anomaly.incident_hold is false for the affected tenant unless long-term hold is approved.
- runbook invocation evidence is attached to evidence/incidents/$INCIDENT_ID.json.
- service owner acknowledged final handoff in #inc-compliance-regulatory.

## Postmortem Template
Use this exact skeleton for the incident document. The field names are intentionally stable for ADR-0263 audit emission extraction.
```markdown
---
doc_class: IncidentPostmortem
runbook_id: compliance-phi-access-anomaly
microservice: compliance
event_class: EVT-COMPLIANCE-PHI_ACCESS_ANOMALY-INCIDENT
incident_id: <INC-...>
severity: sev0
status: draft
detected_at: <UTC>
mitigated_at: <UTC>
resolved_at: <UTC>
commander: <handle>
evidence_hash: <sha256>
---

# PHI Access Anomaly postmortem

## Summary
- What happened in compliance/phi-access-anomaly.
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
- Emit EVT-COMPLIANCE-PHI_ACCESS_ANOMALY-INCIDENT with incident_id, tenant_id, cell_id, principal_id, decision_id, evidence_hash, operator_id, runbook_id.
- Attach dashboard snapshot URLs and command transcripts.
- Seal mitigation and resolution events before closure.

## Corrective Actions
- Action, owner, due date, validation command, linked issue.
```

## Escalation Path
- Primary on-call: PagerDuty oya-compliance-primary; DPO escalation bridge; legal-duty officer.
- Incident SLA: ack 3m for sev0/sev1, 10m for sev2, regulator clock checkpoint every 15m.
- Incident commander: first responder from axis-compliance + DPO office + ops-security; transfer only by explicit message in #inc-compliance-regulatory.
- Security escalation: page `ops-security-primary` immediately for sev0, data-boundary, credential, or audit-seal symptoms.
- Compliance escalation: page `dpo-office-duty` when tenant data, regulator evidence, or breach clock symptoms are present.
- Architecture escalation: page `council-architecture-reviewer` before manual bypass, policy rollback, or invariant relaxation.
- External vendors: Sigstore support; SeaweedFS support; external auditor portal support; regulator submission portal desk. Open a ticket once local dependency health is proven and vendor dependency remains suspect.
- Customer communications: use status page component `oyatie-compliance-phi-access-anomaly` and keep private details in the incident channel.
- Regulatory clock: if any tenant data exposure is possible, start the compliance 72h assessment timer even if exposure is unconfirmed.
- Executive notice: sev0 or fleet-wide sev1 goes to `#exec-incident-readout` within 30 minutes.

## Cross-µservice Coordination
- Notify `identity`: `oya incident handoff --target identity --source compliance --runbook phi-access-anomaly --incident $INCIDENT_ID --severity sev0 --branch A`; expect `202 accepted`.
- Require `identity` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `tenancy`: `oya incident handoff --target tenancy --source compliance --runbook phi-access-anomaly --incident $INCIDENT_ID --severity sev0 --branch B`; expect `202 accepted`.
- Require `tenancy` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `governance`: `oya incident handoff --target governance --source compliance --runbook phi-access-anomaly --incident $INCIDENT_ID --severity sev0 --branch C`; expect `202 accepted`.
- Require `governance` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `observability`: `oya incident handoff --target observability --source compliance --runbook phi-access-anomaly --incident $INCIDENT_ID --severity sev0 --branch D`; expect `202 accepted`.
- Require `observability` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Observability handoff API: `oya incident handoff --target observability --source compliance --runbook phi-access-anomaly --incident $INCIDENT_ID`.
- Governance handoff API: `oya incident handoff --target governance --source compliance --runbook phi-access-anomaly --incident $INCIDENT_ID`.
- Compliance handoff API: `oya incident handoff --target compliance --source compliance --runbook phi-access-anomaly --incident $INCIDENT_ID`.
- Identity handoff API: `oya incident handoff --target identity --source compliance --runbook phi-access-anomaly --incident $INCIDENT_ID`.
- Tenancy handoff API: `oya incident handoff --target tenancy --source compliance --runbook phi-access-anomaly --incident $INCIDENT_ID`.

## Handoff Notes
- Do not hand off with only the alert name; include oya_compliance_phi_access_anomaly_error_ratio, oya_compliance_phi_access_anomaly_lag_seconds, oya_compliance_phi_access_anomaly_queue_depth, current breaker state, and audit seal status.
- Keep compliance-phi-access-anomaly-circuit-breaker owner as axis-compliance + DPO office + ops-security until the receiving service explicitly accepts.
- If another runbook owns the downstream fix, link this incident as upstream and keep this runbook open until downstream verification returns green.
- Close only after EVT-COMPLIANCE-PHI_ACCESS_ANOMALY-INCIDENT has a sealed resolution row and every coordination endpoint above has either accepted or explicitly declined scope.

## Sources Checked During This Substance Pass
- `compliance/dashboards/` for dashboard names and operational panels.
- `compliance/observability/slos/` for OpenSLO alert vocabulary and threshold alignment.
- `compliance/policy/` for named policy and authorization surfaces.
- `compliance/catalog/` for component and owner vocabulary.
- Existing thin runbook topic `phi-access-anomaly` was preserved as the scenario anchor while replacing generic steps with concrete commands.
