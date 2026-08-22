---
doc_class: Runbook
title: Jwt Key Rotation
status: Accepted
date: 2026-05-20
microservice: tenancy
severity: sev2
audience: oncall-engineer
owner_team: axis-tenancy + ops-sre-reliability + ops-security
source_wave: codex-runbooks-substrate-w1
change_scope: substance rewrite of thin existing runbook
doc_status: published
---

# Runbook: Jwt Key Rotation

## Operator Contract
- Runbook id: tenancy-jwt-key-rotation.
- Primary service namespace: `tenancy`.
- Owning rotation: PagerDuty tenancy-primary; data-boundary security secondary.
- Incident channel: `#inc-tenancy-boundary`.
- External dependencies: Citus Data support; Oracle PostgreSQL support; Cloudflare Zero Trust support.
- API authority: `https://tenancy.internal.oyatie.dev/v1/tenancy/jwt-key-rotation/incident-handoff`.
- Audit event class: `EVT-TENANCY-JWT_KEY_ROTATION-INCIDENT` with ADR-0263 fields `incident_id`, `tenant_id`, `cell_id`, `microservice`, `runbook_id`, `decision_id`, `evidence_hash`, `operator_id`.
- Stop condition: mitigation has held for 30 minutes, TenancyJwtKeyRotationCritical is green, and all handoff APIs in Cross-µservice Coordination return `202 accepted`.
- Safety invariant: never clear the incident until `EVT-TENANCY-JWT_KEY_ROTATION-INCIDENT` is sealed and the postmortem skeleton exists under `evidence/postmortems/tenancy-jwt-key-rotation-<incident-id>.md`.

## Trigger Conditions
- Page on alert `TenancyJwtKeyRotationCritical` when `tenancy_jwt_key_rotation_error_ratio > 0.02` for 10 minutes in any production cell.
- Page on alert `TenancyJwtKeyRotationSloBurn` when `tenancy_jwt_key_rotation_lag_seconds > 300` for 2 consecutive evaluator windows.
- Open a sev0 if `tenancy_jwt_key_rotation_correctness_ratio < 0.9999` and the affected label set includes `tenant_id` or `principal_id`.
- Open a sev1 if `tenancy_jwt_key_rotation_queue_depth > 5000` for 15 minutes or retry backlog grows by more than 20 percent in one 5 minute window.
- Trigger from customer report when Support tags the case `tenancy.jwt-key-rotation.customer_visible` in Zendesk.
- Trigger from CI when `cargo run -p dev-cli -- gate validate tenancy-jwt-key-rotation --production-snapshot` exits non-zero against the latest production evidence bundle.
- Primary dashboard: `https://grafana.dev.oyatie.internal/d/tenancy-substrate/jwt-key-rotation?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=117`.
- Secondary dashboard: `https://grafana.dev.oyatie.internal/d/tenancy-substrate/jwt-key-rotation?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=210`.
- Loki explorer: `https://grafana.dev.oyatie.internal/explore?query={namespace="tenancy",runbook="jwt-key-rotation"}`.
- Alertmanager route: `oyatie-tenancy-jwt-key-rotation-critical`; silence only with incident commander approval and `EVT-TENANCY-JWT_KEY_ROTATION-INCIDENT` evidence.
- Synthetic probe: `oya ops probe tenancy jwt-key-rotation --cell prod-us-east-1 --tenant synthetic-canary` returns `healthy=true`.
- Drift detector: `registry/tenancy/jwt-key-rotation/expected-state.json` hash differs from live `https://tenancy.internal.oyatie.dev/v1/admin/state-hash`.

## Symptoms
- User-facing impact: jwt key rotation blocks or corrupts the tenancy control path for affected tenants.
- Operators see Grafana panel `tenant-lifecycle-ops / Jwt Key Rotation burn rate` turn red before the primary alert resolves.
- Loki signature `tenancy.jwt_key_rotation.incident_state=failed` appears with fields `incident_id`, `tenant_id`, `cell_id`, `decision_id`, `evidence_hash`.
- Kubernetes events include `reason=TenancyJwtKeyRotationDegraded` on deployment `tenancy-jwt-key-rotation-worker`.
- Audit-chain shows missing or delayed `EVT-TENANCY-JWT_KEY_ROTATION-INCIDENT` entries when queried with `oya audit-chain query --event-class EVT-TENANCY-JWT_KEY_ROTATION-INCIDENT --since 30m`.
- Metric pattern: `tenancy_jwt_key_rotation_error_ratio` rises before `tenancy_jwt_key_rotation_lag_seconds`; if lag rises first, suspect dependency saturation rather than local regression.
- Metric pattern: `tenancy_jwt_key_rotation_queue_depth` increases while pod CPU stays below 40 percent; suspect downstream refusal or feature flag deadlock.
- Tenant-specific shape: one `tenant_id` dominates labels in `tenancy_jwt_key_rotation_queue_depth`; isolate before fleet mitigation.
- Fleet-wide shape: at least three cells report `TenancyJwtKeyRotationCritical` in one 15 minute window; switch to sev1 bridge even if individual tenants are low-volume.
- Log signature `decision=deny reason=jwt-key-rotation.policy_guard` means the guard is working; investigate caller inputs before rollback.
- Log signature `decision=permit reason=jwt-key-rotation.break_glass` means manual intervention is active; confirm two-person authorization.
- Log signature `audit_emit_status=stalled event_class=EVT-TENANCY-JWT_KEY_ROTATION-INCIDENT` means mitigation cannot be closed until replay succeeds.

## Diagnostic Steps
1. Set incident variables: `export INCIDENT_ID=INC-tenancy-jwt-key-rotation-$(date -u +%Y%m%dT%H%M%SZ); export CELL=prod-us-east-1; export TENANT=synthetic-canary`.
2. Confirm active alerts: `curl -s https://tenancy.internal.oyatie.dev/v1/alerts?runbook=jwt-key-rotation | jq .alerts`.
3. Check Kubernetes rollout: `kubectl -n tenancy rollout status deploy/tenancy-jwt-key-rotation-worker --timeout=60s`.
4. List unhealthy pods: `kubectl -n tenancy get pods -l app=tenancy-jwt-key-rotation -o wide`.
5. Read structured logs: `kubectl -n tenancy logs deploy/tenancy-jwt-key-rotation-worker --since=30m | rg "tenancy.jwt_key_rotation.incident_state|TenancyJwtKeyRotationCritical|EVT-TENANCY-JWT_KEY_ROTATION-INCIDENT"`.
6. Query Loki directly: `logcli query '{namespace="tenancy",runbook="jwt-key-rotation"}' --since=30m --limit=200`.
7. Check Prometheus fast burn: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=tenancy_jwt_key_rotation_error_ratio{cell="prod-us-east-1"}'`.
8. Check lag: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=tenancy_jwt_key_rotation_lag_seconds{cell="prod-us-east-1"}'`.
9. Check queue: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=tenancy_jwt_key_rotation_queue_depth{cell="prod-us-east-1"}'`.
10. Open primary dashboard: `open "https://grafana.dev.oyatie.internal/d/tenancy-substrate/jwt-key-rotation?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=117&var-incident=$INCIDENT_ID"`.
11. Open secondary dashboard: `open "https://grafana.dev.oyatie.internal/d/tenancy-substrate/jwt-key-rotation?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=210&var-tenant=$TENANT"`.
12. Verify audit-chain emission: `oya audit-chain query --event-class EVT-TENANCY-JWT_KEY_ROTATION-INCIDENT --since 30m --cell $CELL --tenant $TENANT`.
13. Verify service state: `oya ops tenancy jwt-key-rotation status --cell $CELL --tenant $TENANT --output json`.
14. Run production snapshot gate: `cargo run -p dev-cli -- gate validate tenancy-jwt-key-rotation --production-snapshot --cell $CELL`.
15. Check Cargo owner crate: `cargo test -p tenancy-domain jwt_key_rotation -- --nocapture`.
16. Check API contract smoke: `curl -s https://tenancy.internal.oyatie.dev/v1/tenancy/jwt-key-rotation/incident-handoff -H "x-tenant: $TENANT"`.
17. Inspect config: `kubectl -n tenancy get configmap tenancy-jwt-key-rotation-config -o yaml`.
18. Inspect feature flags: `oya flags get oya.tenancy.jwt_key_rotation.incident_hold --cell $CELL --tenant $TENANT --output yaml`.
19. Inspect circuit breaker: `oya ops breaker status tenancy-jwt-key-rotation-circuit-breaker --cell $CELL --tenant $TENANT`.
20. Check recent deploy: `kubectl -n tenancy rollout history deploy/tenancy-jwt-key-rotation-worker | tail -20`.
21. Check policy file: `test -f microservices/tenancy/policy/rls-isolation.cedar || test -f microservices/tenancy/policy/rls-isolation.md`.
22. Check SLO files: `ls microservices/tenancy/slos/*.openslo.yaml | sort`.
23. Check catalog components: `find microservices/tenancy/catalog -maxdepth 1 -type f | sort | rg "tenancy|jwt"`.
24. Confirm no cross-cell spread: `oya ops cells query --metric tenancy_jwt_key_rotation_error_ratio --window 30m --threshold 0.02`.
25. Snapshot evidence: `oya evidence snapshot --incident $INCIDENT_ID --microservice tenancy --runbook jwt-key-rotation --output evidence/incidents/$INCIDENT_ID.json`.

### Diagnostic Decision Tree
```text
Jwt Key Rotation incident decision tree
1. Is TenancyJwtKeyRotationCritical firing in more than one cell?
   |-- yes: declare fleet incident, page PagerDuty tenancy-primary; data-boundary security secondary, and run cross-cell containment.
   |-- no: keep scope to the affected cell and continue tenant isolation checks.
2. Does tenancy_jwt_key_rotation_queue_depth grow while tenancy_jwt_key_rotation_error_ratio is flat?
   |-- yes: downstream dependency or replay backlog; choose mitigation branch B.
   |-- no: local regression or bad input; continue branch selection.
3. Does audit-chain show EVT-TENANCY-JWT_KEY_ROTATION-INCIDENT gaps?
   |-- yes: do not close; run evidence replay before resolution.
   |-- no: mitigation can proceed after state is green.
4. Is customer or regulator impact confirmed?
   |-- yes: promote severity, open #inc-tenancy-boundary, and notify compliance handoff.
   |-- no: keep internal incident and collect evidence.
```
- Branch A (policy or key mismatch): use the matching mitigation block below and record `decision_branch=A` in `EVT-TENANCY-JWT_KEY_ROTATION-INCIDENT`.
- Branch B (rollback is safe and bounded): use the matching mitigation block below and record `decision_branch=B` in `EVT-TENANCY-JWT_KEY_ROTATION-INCIDENT`.
- Branch C (rollback would widen access): use the matching mitigation block below and record `decision_branch=C` in `EVT-TENANCY-JWT_KEY_ROTATION-INCIDENT`.
- Branch D (manual two-person approval required): use the matching mitigation block below and record `decision_branch=D` in `EVT-TENANCY-JWT_KEY_ROTATION-INCIDENT`.

## Mitigation Steps
1. Acknowledge page: `pd incident ack --service tenancy --incident $INCIDENT_ID`.
2. Create bridge: `oya incident bridge create --incident $INCIDENT_ID --channel #inc-tenancy-boundary --severity sev2`.
3. Freeze risky automation: `oya flags set oya.tenancy.jwt_key_rotation.incident_hold=true --cell $CELL --tenant $TENANT --reason $INCIDENT_ID`.
4. Enable circuit breaker: `oya ops breaker open tenancy-jwt-key-rotation-circuit-breaker --cell $CELL --tenant $TENANT --ttl 30m --reason $INCIDENT_ID`.
5. Reduce blast radius: `kubectl -n tenancy scale deploy/tenancy-jwt-key-rotation-worker --replicas=1`.
6. Protect tenant boundary: `oya tenancy quarantine --tenant $TENANT --reason tenancy-jwt-key-rotation --ttl 60m`.
7. Pause promotion: incident hold PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
8. Drain queue safely: `oya ops tenancy jwt-key-rotation drain --cell $CELL --tenant $TENANT --max-items 500 --dry-run`.
9. Execute bounded drain: `oya ops tenancy jwt-key-rotation drain --cell $CELL --tenant $TENANT --max-items 500 --confirm $INCIDENT_ID`.
10. Replay missing audit events: `oya audit-chain replay --event-class EVT-TENANCY-JWT_KEY_ROTATION-INCIDENT --incident $INCIDENT_ID --from evidence/incidents/$INCIDENT_ID.json`.
11. Rollback last deploy if causal: `kubectl -n tenancy rollout undo deploy/tenancy-jwt-key-rotation-worker`.
12. Raise HPA cap if saturation: `kubectl -n tenancy patch hpa tenancy-jwt-key-rotation-worker --type merge -p '{"spec":{"maxReplicas":12}}'`.
13. Throttle hot tenant: `oya ops rate-limit set --tenant $TENANT --surface tenancy.jwt-key-rotation --rps 25 --ttl 30m`.
14. Block abusive principal: `oya identity principal suspend --principal suspected-abuse --tenant $TENANT --reason $INCIDENT_ID`.
15. Protect evidence: `oya evidence freeze --incident $INCIDENT_ID --paths microservices/tenancy/runbooks/jwt-key-rotation.md,evidence/incidents/$INCIDENT_ID.json`.
16. Notify service owners: `oya notify service-owner --microservice tenancy --incident $INCIDENT_ID --channel #inc-tenancy-boundary`.
17. Open external vendor ticket: `oya vendor ticket open --vendor primary-tenancy --incident $INCIDENT_ID --summary jwt-key-rotation`.
18. Confirm breaker effect: `oya ops breaker status tenancy-jwt-key-rotation-circuit-breaker --cell $CELL --tenant $TENANT --expect open`.
19. Confirm user impact reduced: `curl -s https://tenancy.internal.oyatie.dev/v1/tenancy/jwt-key-rotation/incident-handoff/health -H "x-tenant: $TENANT"`.
20. Emit mitigation audit: `oya audit-chain emit --event-class EVT-TENANCY-JWT_KEY_ROTATION-INCIDENT --incident $INCIDENT_ID --field mitigation=active --field runbook=jwt-key-rotation`.

### Mitigation Branch Guidance
- Branch A: policy or key mismatch.
  - Required action: keep `tenancy-jwt-key-rotation-circuit-breaker` open until `tenancy_jwt_key_rotation_error_ratio` is below 0.005 for 3 windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/tenancy-substrate/jwt-key-rotation?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=117` to the incident.
  - Required audit: emit `EVT-TENANCY-JWT_KEY_ROTATION-INCIDENT` with `branch=A`, `operator_id`, and `evidence_hash`.
- Branch B: rollback is safe and bounded.
  - Required action: keep `tenancy-jwt-key-rotation-circuit-breaker` open until `tenancy_jwt_key_rotation_error_ratio` is below 0.005 for 3 windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/tenancy-substrate/jwt-key-rotation?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=118` to the incident.
  - Required audit: emit `EVT-TENANCY-JWT_KEY_ROTATION-INCIDENT` with `branch=B`, `operator_id`, and `evidence_hash`.
- Branch C: rollback would widen access.
  - Required action: keep `tenancy-jwt-key-rotation-circuit-breaker` open until `tenancy_jwt_key_rotation_error_ratio` is below 0.005 for 3 windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/tenancy-substrate/jwt-key-rotation?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=119` to the incident.
  - Required audit: emit `EVT-TENANCY-JWT_KEY_ROTATION-INCIDENT` with `branch=C`, `operator_id`, and `evidence_hash`.
- Branch D: manual two-person approval required.
  - Required action: keep `tenancy-jwt-key-rotation-circuit-breaker` open until `tenancy_jwt_key_rotation_error_ratio` is below 0.005 for 3 windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/tenancy-substrate/jwt-key-rotation?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=120` to the incident.
  - Required audit: emit `EVT-TENANCY-JWT_KEY_ROTATION-INCIDENT` with `branch=D`, `operator_id`, and `evidence_hash`.

## Resolution Steps
1. Identify code owner path: `rg "jwt_key_rotation|TenancyJwtKeyRotationCritical|tenancy.jwt_key_rotation.incident_state" crates microservices/tenancy -g "!microservices/tenancy/runbooks/**"`.
2. Patch domain invariant: `edit tenancy-domain where jwt_key_rotation state transition is validated`.
3. Patch API guard: `edit microservices/tenancy/contracts/openapi.yaml or catalog REST binding if the failing path is north-south`.
4. Patch policy: `edit microservices/tenancy/policy/rls-isolation.cedar or .md with explicit deny/permit branch`.
5. Patch runtime config: `edit microservices/tenancy/iac/k8s-deployment.yaml or secret-bindings.yaml if deploy/config drift caused the incident`.
6. Add regression test: `cargo test -p tenancy-domain jwt_key_rotation_incident_regression -- --nocapture`.
7. Add gate evidence: `cargo run -p dev-cli -- gate validate tenancy-jwt-key-rotation --fixture incident-jwt-key-rotation.json`.
8. Add SLO assertion: `update microservices/tenancy/slos/* with alert TenancyJwtKeyRotationCritical when this was a missing alert`.
9. Add dashboard panel: `update microservices/tenancy/dashboards/tenant-lifecycle-ops.json with tenancy_jwt_key_rotation_error_ratio, tenancy_jwt_key_rotation_lag_seconds, and tenancy_jwt_key_rotation_queue_depth`.
10. Rebuild affected crate: `cargo check -p tenancy-domain --all-targets`.
11. Run targeted tests: `cargo test -p tenancy-domain --all-features`.
12. Run policy validation: `cargo run -p dev-cli -- gate validate tenancy-policy --microservice tenancy`.
13. Deploy canary: `oya deploy canary --microservice tenancy --component jwt-key-rotation-worker --cell $CELL --weight 1`.
14. Watch burn rate: `oya ops watch --metric tenancy_jwt_key_rotation_error_ratio --threshold 0.005 --window 30m --cell $CELL`.
15. Close circuit breaker: `oya ops breaker close tenancy-jwt-key-rotation-circuit-breaker --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
16. Unfreeze automation: `oya flags set oya.tenancy.jwt_key_rotation.incident_hold=false --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
17. Resume promotion: recovery PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
18. Seal resolution audit: `oya audit-chain emit --event-class EVT-TENANCY-JWT_KEY_ROTATION-INCIDENT --incident $INCIDENT_ID --field resolution=complete --field runbook=jwt-key-rotation`.
19. Verify seal: `oya audit-chain verify --event-class EVT-TENANCY-JWT_KEY_ROTATION-INCIDENT --incident $INCIDENT_ID`.
20. Attach final evidence: `oya evidence attach --incident $INCIDENT_ID --file evidence/incidents/$INCIDENT_ID.json --kind final-resolution`.

### Code Paths To Inspect First
- `tenancy-domain`: inspect for jwt_key_rotation invariants, alert emission, and ADR-0263 evidence fields before touching adjacent code path 1.
- `tenancy-kernel`: inspect for jwt_key_rotation invariants, alert emission, and ADR-0263 evidence fields before touching adjacent code path 2.
- `tenancy-api`: inspect for jwt_key_rotation invariants, alert emission, and ADR-0263 evidence fields before touching adjacent code path 3.
- `microservices/tenancy/contracts/`: verify this surface only when the incident evidence points there.
- `microservices/tenancy/dashboards/tenant-lifecycle-ops.json`: verify this surface only when the incident evidence points there.
- `microservices/tenancy/slos/`: verify this surface only when the incident evidence points there.
- `microservices/tenancy/policy/rls-isolation.*`: verify this surface only when the incident evidence points there.

## Verification Checklist
- TenancyJwtKeyRotationCritical and TenancyJwtKeyRotationSloBurn are both resolved in Alertmanager for 30 minutes.
- tenancy_jwt_key_rotation_error_ratio < 0.005 for 3 consecutive 10 minute windows.
- tenancy_jwt_key_rotation_lag_seconds < 120 for all production cells.
- tenancy_jwt_key_rotation_queue_depth is draining and not growing for the affected tenant.
- dashboard https://grafana.dev.oyatie.internal/d/tenancy-substrate/jwt-key-rotation?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=117 shows green panels for the affected cell.
- audit-chain query for EVT-TENANCY-JWT_KEY_ROTATION-INCIDENT returns mitigation and resolution events.
- circuit breaker tenancy-jwt-key-rotation-circuit-breaker is closed after rollback window.
- feature flag oya.tenancy.jwt_key_rotation.incident_hold is false for the affected tenant unless long-term hold is approved.
- runbook invocation evidence is attached to evidence/incidents/$INCIDENT_ID.json.
- service owner acknowledged final handoff in #inc-tenancy-boundary.

## Postmortem Template
Use this exact skeleton for the incident document. The field names are intentionally stable for ADR-0263 audit emission extraction.
```markdown
---
doc_class: IncidentPostmortem
runbook_id: tenancy-jwt-key-rotation
microservice: tenancy
event_class: EVT-TENANCY-JWT_KEY_ROTATION-INCIDENT
incident_id: <INC-...>
severity: sev2
status: draft
detected_at: <UTC>
mitigated_at: <UTC>
resolved_at: <UTC>
commander: <handle>
evidence_hash: <sha256>
---

# Jwt Key Rotation postmortem

## Summary
- What happened in tenancy/jwt-key-rotation.
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
- Emit EVT-TENANCY-JWT_KEY_ROTATION-INCIDENT with incident_id, tenant_id, cell_id, principal_id, decision_id, evidence_hash, operator_id, runbook_id.
- Attach dashboard snapshot URLs and command transcripts.
- Seal mitigation and resolution events before closure.

## Corrective Actions
- Action, owner, due date, validation command, linked issue.
```

## Escalation Path
- Primary on-call: PagerDuty tenancy-primary; data-boundary security secondary.
- Incident SLA: ack 3m for sev0/sev1, 10m for sev2, isolation checkpoint every 10m until contained.
- Incident commander: first responder from axis-tenancy + ops-sre-reliability + ops-security; transfer only by explicit message in #inc-tenancy-boundary.
- Security escalation: page `ops-security-primary` immediately for sev0, data-boundary, credential, or audit-seal symptoms.
- Compliance escalation: page `dpo-office-duty` when tenant data, regulator evidence, or breach clock symptoms are present.
- Architecture escalation: page `council-architecture-reviewer` before manual bypass, policy rollback, or invariant relaxation.
- External vendors: Citus Data support; Oracle PostgreSQL support; Cloudflare Zero Trust support. Open a ticket once local dependency health is proven and vendor dependency remains suspect.
- Customer communications: use status page component `oyatie-tenancy-jwt-key-rotation` and keep private details in the incident channel.
- Regulatory clock: if any tenant data exposure is possible, start the compliance 72h assessment timer even if exposure is unconfirmed.
- Executive notice: sev0 or fleet-wide sev1 goes to `#exec-incident-readout` within 30 minutes.

## Cross-µservice Coordination
- Notify `identity`: `oya incident handoff --target identity --source tenancy --runbook jwt-key-rotation --incident $INCIDENT_ID --severity sev2 --branch A`; expect `202 accepted`.
- Require `identity` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `compliance`: `oya incident handoff --target compliance --source tenancy --runbook jwt-key-rotation --incident $INCIDENT_ID --severity sev2 --branch B`; expect `202 accepted`.
- Require `compliance` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `governance`: `oya incident handoff --target governance --source tenancy --runbook jwt-key-rotation --incident $INCIDENT_ID --severity sev2 --branch C`; expect `202 accepted`.
- Require `governance` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `observability`: `oya incident handoff --target observability --source tenancy --runbook jwt-key-rotation --incident $INCIDENT_ID --severity sev2 --branch D`; expect `202 accepted`.
- Require `observability` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Observability handoff API: `oya incident handoff --target observability --source tenancy --runbook jwt-key-rotation --incident $INCIDENT_ID`.
- Governance handoff API: `oya incident handoff --target governance --source tenancy --runbook jwt-key-rotation --incident $INCIDENT_ID`.
- Compliance handoff API: `oya incident handoff --target compliance --source tenancy --runbook jwt-key-rotation --incident $INCIDENT_ID`.
- Identity handoff API: `oya incident handoff --target identity --source tenancy --runbook jwt-key-rotation --incident $INCIDENT_ID`.
- Tenancy handoff API: `oya incident handoff --target tenancy --source tenancy --runbook jwt-key-rotation --incident $INCIDENT_ID`.

## Handoff Notes
- Do not hand off with only the alert name; include tenancy_jwt_key_rotation_error_ratio, tenancy_jwt_key_rotation_lag_seconds, tenancy_jwt_key_rotation_queue_depth, current breaker state, and audit seal status.
- Keep tenancy-jwt-key-rotation-circuit-breaker owner as axis-tenancy + ops-sre-reliability + ops-security until the receiving service explicitly accepts.
- If another runbook owns the downstream fix, link this incident as upstream and keep this runbook open until downstream verification returns green.
- Close only after EVT-TENANCY-JWT_KEY_ROTATION-INCIDENT has a sealed resolution row and every coordination endpoint above has either accepted or explicitly declined scope.

## Sources Checked During This Substance Pass
- `microservices/tenancy/dashboards/` for dashboard names and operational panels.
- `microservices/tenancy/slos/` for OpenSLO alert vocabulary and threshold alignment.
- `microservices/tenancy/policy/` for named policy and authorization surfaces.
- `microservices/tenancy/catalog/` for component and owner vocabulary.
- Existing thin runbook topic `jwt-key-rotation` was preserved as the scenario anchor while replacing generic steps with concrete commands.
