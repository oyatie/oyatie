---
doc_class: Runbook
title: Recovery Key Mass Issue Investigation
status: Accepted
date: 2026-05-20
microservice: identity
severity: sev0
audience: security-engineer
owner_team: axis-identity + ops-security
source_wave: codex-runbooks-substrate-w1
change_scope: net-new critical operational scenario
doc_status: published
---

# Runbook: Recovery Key Mass Issue Investigation

## Operator Contract
- Runbook id: identity-recovery-key-mass-issue-investigation.
- Primary service namespace: `identity`.
- Owning rotation: PagerDuty oya-identity-primary; ops-security secondary.
- Incident channel: `#inc-identity-security`.
- External dependencies: Zitadel support; Yubico enterprise support; WebAuthn metadata service desk.
- API authority: `https://identity.internal.oyatie.dev/v1/identity/recovery-key-mass-issue-investigation/incident-handoff`.
- Audit event class: `EVT-IDENTITY-RECOVERY_KEY_MASS_ISSUE_INVESTIGATION-INCIDENT` with ADR-0263 fields `incident_id`, `tenant_id`, `cell_id`, `microservice`, `runbook_id`, `decision_id`, `evidence_hash`, `operator_id`.
- Stop condition: mitigation has held for 30 minutes, IdentityRecoveryKeyMassIssueInvestigationCritical is green, and all handoff APIs in Cross-µservice Coordination return `202 accepted`.
- Safety invariant: never clear the incident until `EVT-IDENTITY-RECOVERY_KEY_MASS_ISSUE_INVESTIGATION-INCIDENT` is sealed and the postmortem skeleton exists under `evidence/postmortems/identity-recovery-key-mass-issue-investigation-<incident-id>.md`.

## Trigger Conditions
- Page on alert `IdentityRecoveryKeyMassIssueInvestigationCritical` when `oya_identity_recovery_key_mass_issue_investigation_error_ratio > 0.02` for 10 minutes in any production cell.
- Page on alert `IdentityRecoveryKeyMassIssueInvestigationSloBurn` when `oya_identity_recovery_key_mass_issue_investigation_lag_seconds > 300` for 2 consecutive evaluator windows.
- Open a sev0 if `oya_identity_recovery_key_mass_issue_investigation_correctness_ratio < 0.9999` and the affected label set includes `tenant_id` or `principal_id`.
- Open a sev1 if `oya_identity_recovery_key_mass_issue_investigation_queue_depth > 5000` for 15 minutes or retry backlog grows by more than 20 percent in one 5 minute window.
- Trigger from customer report when Support tags the case `identity.recovery-key-mass-issue-investigation.customer_visible` in Zendesk.
- Trigger from CI when `cargo run -p oya-dev-cli -- gate validate identity-recovery-key-mass-issue-investigation --production-snapshot` exits non-zero against the latest production evidence bundle.
- Primary dashboard: `https://grafana.dev.oyatie.internal/d/identity-substrate/recovery-key-mass-issue-investigation?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=104`.
- Secondary dashboard: `https://grafana.dev.oyatie.internal/d/identity-substrate/recovery-key-mass-issue-investigation?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=216`.
- Loki explorer: `https://grafana.dev.oyatie.internal/explore?query={namespace="identity",runbook="recovery-key-mass-issue-investigation"}`.
- Alertmanager route: `oyatie-identity-recovery-key-mass-issue-investigation-critical`; silence only with incident commander approval and `EVT-IDENTITY-RECOVERY_KEY_MASS_ISSUE_INVESTIGATION-INCIDENT` evidence.
- Synthetic probe: `oya ops probe identity recovery-key-mass-issue-investigation --cell prod-us-east-1 --tenant synthetic-canary` returns `healthy=true`.
- Drift detector: `registry/identity/recovery-key-mass-issue-investigation/expected-state.json` hash differs from live `https://identity.internal.oyatie.dev/v1/admin/state-hash`.

## Symptoms
- User-facing impact: recovery key mass issue investigation blocks or corrupts the identity control path for affected tenants.
- Operators see Grafana panel `scim-provisioning-health / Recovery Key Mass Issue Investigation burn rate` turn red before the primary alert resolves.
- Loki signature `identity.recovery_key_mass_issue_investigation.incident_state=failed` appears with fields `incident_id`, `tenant_id`, `cell_id`, `decision_id`, `evidence_hash`.
- Kubernetes events include `reason=IdentityRecoveryKeyMassIssueInvestigationDegraded` on deployment `identity-recovery-key-mass-issue-investigation-worker`.
- Audit-chain shows missing or delayed `EVT-IDENTITY-RECOVERY_KEY_MASS_ISSUE_INVESTIGATION-INCIDENT` entries when queried with `oya audit-chain query --event-class EVT-IDENTITY-RECOVERY_KEY_MASS_ISSUE_INVESTIGATION-INCIDENT --since 30m`.
- Metric pattern: `oya_identity_recovery_key_mass_issue_investigation_error_ratio` rises before `oya_identity_recovery_key_mass_issue_investigation_lag_seconds`; if lag rises first, suspect dependency saturation rather than local regression.
- Metric pattern: `oya_identity_recovery_key_mass_issue_investigation_queue_depth` increases while pod CPU stays below 40 percent; suspect downstream refusal or feature flag deadlock.
- Tenant-specific shape: one `tenant_id` dominates labels in `oya_identity_recovery_key_mass_issue_investigation_queue_depth`; isolate before fleet mitigation.
- Fleet-wide shape: at least three cells report `IdentityRecoveryKeyMassIssueInvestigationCritical` in one 15 minute window; switch to sev1 bridge even if individual tenants are low-volume.
- Log signature `decision=deny reason=recovery-key-mass-issue-investigation.policy_guard` means the guard is working; investigate caller inputs before rollback.
- Log signature `decision=permit reason=recovery-key-mass-issue-investigation.break_glass` means manual intervention is active; confirm two-person authorization.
- Log signature `audit_emit_status=stalled event_class=EVT-IDENTITY-RECOVERY_KEY_MASS_ISSUE_INVESTIGATION-INCIDENT` means mitigation cannot be closed until replay succeeds.

## Diagnostic Steps
1. Set incident variables: `export INCIDENT_ID=INC-identity-recovery-key-mass-issue-investigation-$(date -u +%Y%m%dT%H%M%SZ); export CELL=prod-us-east-1; export TENANT=synthetic-canary`.
2. Confirm active alerts: `curl -s https://identity.internal.oyatie.dev/v1/alerts?runbook=recovery-key-mass-issue-investigation | jq .alerts`.
3. Check Kubernetes rollout: `kubectl -n identity rollout status deploy/identity-recovery-key-mass-issue-investigation-worker --timeout=60s`.
4. List unhealthy pods: `kubectl -n identity get pods -l app=identity-recovery-key-mass-issue-investigation -o wide`.
5. Read structured logs: `kubectl -n identity logs deploy/identity-recovery-key-mass-issue-investigation-worker --since=30m | rg "identity.recovery_key_mass_issue_investigation.incident_state|IdentityRecoveryKeyMassIssueInvestigationCritical|EVT-IDENTITY-RECOVERY_KEY_MASS_ISSUE_INVESTIGATION-INCIDENT"`.
6. Query Loki directly: `logcli query '{namespace="identity",runbook="recovery-key-mass-issue-investigation"}' --since=30m --limit=200`.
7. Check Prometheus fast burn: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_identity_recovery_key_mass_issue_investigation_error_ratio{cell="prod-us-east-1"}'`.
8. Check lag: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_identity_recovery_key_mass_issue_investigation_lag_seconds{cell="prod-us-east-1"}'`.
9. Check queue: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_identity_recovery_key_mass_issue_investigation_queue_depth{cell="prod-us-east-1"}'`.
10. Open primary dashboard: `open "https://grafana.dev.oyatie.internal/d/identity-substrate/recovery-key-mass-issue-investigation?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=104&var-incident=$INCIDENT_ID"`.
11. Open secondary dashboard: `open "https://grafana.dev.oyatie.internal/d/identity-substrate/recovery-key-mass-issue-investigation?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=216&var-tenant=$TENANT"`.
12. Verify audit-chain emission: `oya audit-chain query --event-class EVT-IDENTITY-RECOVERY_KEY_MASS_ISSUE_INVESTIGATION-INCIDENT --since 30m --cell $CELL --tenant $TENANT`.
13. Verify service state: `oya ops identity recovery-key-mass-issue-investigation status --cell $CELL --tenant $TENANT --output json`.
14. Run production snapshot gate: `cargo run -p oya-dev-cli -- gate validate identity-recovery-key-mass-issue-investigation --production-snapshot --cell $CELL`.
15. Check Cargo owner crate: `cargo test -p oya-identity-domain recovery_key_mass_issue_investigation -- --nocapture`.
16. Check API contract smoke: `curl -s https://identity.internal.oyatie.dev/v1/identity/recovery-key-mass-issue-investigation/incident-handoff -H "x-oya-tenant: $TENANT"`.
17. Inspect config: `kubectl -n identity get configmap identity-recovery-key-mass-issue-investigation-config -o yaml`.
18. Inspect feature flags: `oya flags get oya.identity.recovery_key_mass_issue_investigation.incident_hold --cell $CELL --tenant $TENANT --output yaml`.
19. Inspect circuit breaker: `oya ops breaker status identity-recovery-key-mass-issue-investigation-circuit-breaker --cell $CELL --tenant $TENANT`.
20. Check recent deploy: `kubectl -n identity rollout history deploy/identity-recovery-key-mass-issue-investigation-worker | tail -20`.
21. Check policy file: `test -f iam/identity/policy/operator-recovery.cedar || test -f microservices/identity/policy/operator-recovery.md`.
22. Check SLO files: `ls iam/observability/slos/identity/*.openslo.yaml | sort`.
23. Check catalog components: `find iam/identity/catalog -maxdepth 1 -type f | sort | rg "identity|recovery"`.
24. Confirm no cross-cell spread: `oya ops cells query --metric oya_identity_recovery_key_mass_issue_investigation_error_ratio --window 30m --threshold 0.02`.
25. Snapshot evidence: `oya evidence snapshot --incident $INCIDENT_ID --microservice identity --runbook recovery-key-mass-issue-investigation --output evidence/incidents/$INCIDENT_ID.json`.

### Diagnostic Decision Tree
```text
Recovery Key Mass Issue Investigation incident decision tree
1. Is IdentityRecoveryKeyMassIssueInvestigationCritical firing in more than one cell?
   |-- yes: declare fleet incident, page PagerDuty oya-identity-primary; ops-security secondary, and run cross-cell containment.
   |-- no: keep scope to the affected cell and continue tenant isolation checks.
2. Does oya_identity_recovery_key_mass_issue_investigation_queue_depth grow while oya_identity_recovery_key_mass_issue_investigation_error_ratio is flat?
   |-- yes: downstream dependency or replay backlog; choose mitigation branch B.
   |-- no: local regression or bad input; continue branch selection.
3. Does audit-chain show EVT-IDENTITY-RECOVERY_KEY_MASS_ISSUE_INVESTIGATION-INCIDENT gaps?
   |-- yes: do not close; run evidence replay before resolution.
   |-- no: mitigation can proceed after state is green.
4. Is customer or regulator impact confirmed?
   |-- yes: promote severity, open #inc-identity-security, and notify compliance handoff.
   |-- no: keep internal incident and collect evidence.
```
- Branch A (policy or key mismatch): use the matching mitigation block below and record `decision_branch=A` in `EVT-IDENTITY-RECOVERY_KEY_MASS_ISSUE_INVESTIGATION-INCIDENT`.
- Branch B (rollback is safe and bounded): use the matching mitigation block below and record `decision_branch=B` in `EVT-IDENTITY-RECOVERY_KEY_MASS_ISSUE_INVESTIGATION-INCIDENT`.
- Branch C (rollback would widen access): use the matching mitigation block below and record `decision_branch=C` in `EVT-IDENTITY-RECOVERY_KEY_MASS_ISSUE_INVESTIGATION-INCIDENT`.
- Branch D (manual two-person approval required): use the matching mitigation block below and record `decision_branch=D` in `EVT-IDENTITY-RECOVERY_KEY_MASS_ISSUE_INVESTIGATION-INCIDENT`.

## Mitigation Steps
1. Acknowledge page: `pd incident ack --service identity --incident $INCIDENT_ID`.
2. Create bridge: `oya incident bridge create --incident $INCIDENT_ID --channel #inc-identity-security --severity sev0`.
3. Freeze risky automation: `oya flags set oya.identity.recovery_key_mass_issue_investigation.incident_hold=true --cell $CELL --tenant $TENANT --reason $INCIDENT_ID`.
4. Enable circuit breaker: `oya ops breaker open identity-recovery-key-mass-issue-investigation-circuit-breaker --cell $CELL --tenant $TENANT --ttl 30m --reason $INCIDENT_ID`.
5. Reduce blast radius: `kubectl -n identity scale deploy/identity-recovery-key-mass-issue-investigation-worker --replicas=1`.
6. Protect tenant boundary: `oya tenancy quarantine --tenant $TENANT --reason identity-recovery-key-mass-issue-investigation --ttl 60m`.
7. Pause promotion: incident hold PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
8. Drain queue safely: `oya ops identity recovery-key-mass-issue-investigation drain --cell $CELL --tenant $TENANT --max-items 500 --dry-run`.
9. Execute bounded drain: `oya ops identity recovery-key-mass-issue-investigation drain --cell $CELL --tenant $TENANT --max-items 500 --confirm $INCIDENT_ID`.
10. Replay missing audit events: `oya audit-chain replay --event-class EVT-IDENTITY-RECOVERY_KEY_MASS_ISSUE_INVESTIGATION-INCIDENT --incident $INCIDENT_ID --from evidence/incidents/$INCIDENT_ID.json`.
11. Rollback last deploy if causal: `kubectl -n identity rollout undo deploy/identity-recovery-key-mass-issue-investigation-worker`.
12. Raise HPA cap if saturation: `kubectl -n identity patch hpa identity-recovery-key-mass-issue-investigation-worker --type merge -p '{"spec":{"maxReplicas":12}}'`.
13. Throttle hot tenant: `oya ops rate-limit set --tenant $TENANT --surface identity.recovery-key-mass-issue-investigation --rps 25 --ttl 30m`.
14. Block abusive principal: `oya identity principal suspend --principal suspected-abuse --tenant $TENANT --reason $INCIDENT_ID`.
15. Protect evidence: `oya evidence freeze --incident $INCIDENT_ID --paths iam/identity/runbooks/recovery-key-mass-issue-investigation.md,evidence/incidents/$INCIDENT_ID.json`.
16. Notify service owners: `oya notify service-owner --microservice identity --incident $INCIDENT_ID --channel #inc-identity-security`.
17. Open external vendor ticket: `oya vendor ticket open --vendor primary-identity --incident $INCIDENT_ID --summary recovery-key-mass-issue-investigation`.
18. Confirm breaker effect: `oya ops breaker status identity-recovery-key-mass-issue-investigation-circuit-breaker --cell $CELL --tenant $TENANT --expect open`.
19. Confirm user impact reduced: `curl -s https://identity.internal.oyatie.dev/v1/identity/recovery-key-mass-issue-investigation/incident-handoff/health -H "x-oya-tenant: $TENANT"`.
20. Emit mitigation audit: `oya audit-chain emit --event-class EVT-IDENTITY-RECOVERY_KEY_MASS_ISSUE_INVESTIGATION-INCIDENT --incident $INCIDENT_ID --field mitigation=active --field runbook=recovery-key-mass-issue-investigation`.

### Mitigation Branch Guidance
- Branch A: policy or key mismatch.
  - Required action: keep `identity-recovery-key-mass-issue-investigation-circuit-breaker` open until `oya_identity_recovery_key_mass_issue_investigation_error_ratio` is below 0.005 for 3 windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/identity-substrate/recovery-key-mass-issue-investigation?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=104` to the incident.
  - Required audit: emit `EVT-IDENTITY-RECOVERY_KEY_MASS_ISSUE_INVESTIGATION-INCIDENT` with `branch=A`, `operator_id`, and `evidence_hash`.
- Branch B: rollback is safe and bounded.
  - Required action: keep `identity-recovery-key-mass-issue-investigation-circuit-breaker` open until `oya_identity_recovery_key_mass_issue_investigation_error_ratio` is below 0.005 for 3 windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/identity-substrate/recovery-key-mass-issue-investigation?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=105` to the incident.
  - Required audit: emit `EVT-IDENTITY-RECOVERY_KEY_MASS_ISSUE_INVESTIGATION-INCIDENT` with `branch=B`, `operator_id`, and `evidence_hash`.
- Branch C: rollback would widen access.
  - Required action: keep `identity-recovery-key-mass-issue-investigation-circuit-breaker` open until `oya_identity_recovery_key_mass_issue_investigation_error_ratio` is below 0.005 for 3 windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/identity-substrate/recovery-key-mass-issue-investigation?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=106` to the incident.
  - Required audit: emit `EVT-IDENTITY-RECOVERY_KEY_MASS_ISSUE_INVESTIGATION-INCIDENT` with `branch=C`, `operator_id`, and `evidence_hash`.
- Branch D: manual two-person approval required.
  - Required action: keep `identity-recovery-key-mass-issue-investigation-circuit-breaker` open until `oya_identity_recovery_key_mass_issue_investigation_error_ratio` is below 0.005 for 3 windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/identity-substrate/recovery-key-mass-issue-investigation?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=107` to the incident.
  - Required audit: emit `EVT-IDENTITY-RECOVERY_KEY_MASS_ISSUE_INVESTIGATION-INCIDENT` with `branch=D`, `operator_id`, and `evidence_hash`.

## Resolution Steps
1. Identify code owner path: `rg "recovery_key_mass_issue_investigation|IdentityRecoveryKeyMassIssueInvestigationCritical|identity.recovery_key_mass_issue_investigation.incident_state" crates microservices/identity -g "!iam/identity/runbooks/**"`.
2. Patch domain invariant: `edit oya-identity-domain where recovery_key_mass_issue_investigation state transition is validated`.
3. Patch API guard: `edit microservices/identity/contracts/openapi.yaml or catalog REST binding if the failing path is north-south`.
4. Patch policy: `edit iam/identity/policy/operator-recovery.cedar or .md with explicit deny/permit branch`.
5. Patch runtime config: `edit microservices/identity/iac/k8s-deployment.yaml or secret-bindings.yaml if deploy/config drift caused the incident`.
6. Add regression test: `cargo test -p oya-identity-domain recovery_key_mass_issue_investigation_incident_regression -- --nocapture`.
7. Add gate evidence: `cargo run -p oya-dev-cli -- gate validate identity-recovery-key-mass-issue-investigation --fixture incident-recovery-key-mass-issue-investigation.json`.
8. Add SLO assertion: `update iam/observability/slos/identity/* with alert IdentityRecoveryKeyMassIssueInvestigationCritical when this was a missing alert`.
9. Add dashboard panel: `update iam/identity/dashboards/scim-provisioning-health.json with oya_identity_recovery_key_mass_issue_investigation_error_ratio, oya_identity_recovery_key_mass_issue_investigation_lag_seconds, and oya_identity_recovery_key_mass_issue_investigation_queue_depth`.
10. Rebuild affected crate: `cargo check -p oya-identity-domain --all-targets`.
11. Run targeted tests: `cargo test -p oya-identity-domain --all-features`.
12. Run policy validation: `cargo run -p oya-dev-cli -- gate validate identity-policy --microservice identity`.
13. Deploy canary: `oya deploy canary --microservice identity --component recovery-key-mass-issue-investigation-worker --cell $CELL --weight 1`.
14. Watch burn rate: `oya ops watch --metric oya_identity_recovery_key_mass_issue_investigation_error_ratio --threshold 0.005 --window 30m --cell $CELL`.
15. Close circuit breaker: `oya ops breaker close identity-recovery-key-mass-issue-investigation-circuit-breaker --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
16. Unfreeze automation: `oya flags set oya.identity.recovery_key_mass_issue_investigation.incident_hold=false --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
17. Resume promotion: recovery PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
18. Seal resolution audit: `oya audit-chain emit --event-class EVT-IDENTITY-RECOVERY_KEY_MASS_ISSUE_INVESTIGATION-INCIDENT --incident $INCIDENT_ID --field resolution=complete --field runbook=recovery-key-mass-issue-investigation`.
19. Verify seal: `oya audit-chain verify --event-class EVT-IDENTITY-RECOVERY_KEY_MASS_ISSUE_INVESTIGATION-INCIDENT --incident $INCIDENT_ID`.
20. Attach final evidence: `oya evidence attach --incident $INCIDENT_ID --file evidence/incidents/$INCIDENT_ID.json --kind final-resolution`.

### Code Paths To Inspect First
- `oya-identity-domain`: inspect for recovery_key_mass_issue_investigation invariants, alert emission, and ADR-0263 evidence fields before touching adjacent code path 1.
- `oya-cloud-iam-domain`: inspect for recovery_key_mass_issue_investigation invariants, alert emission, and ADR-0263 evidence fields before touching adjacent code path 2.
- `oya-cloud-iam-api`: inspect for recovery_key_mass_issue_investigation invariants, alert emission, and ADR-0263 evidence fields before touching adjacent code path 3.
- `iam/identity/contracts/`: verify this surface only when the incident evidence points there.
- `iam/identity/dashboards/scim-provisioning-health.json`: verify this surface only when the incident evidence points there.
- `iam/observability/slos/identity/`: verify this surface only when the incident evidence points there.
- `microservices/identity/policy/operator-recovery.*`: verify this surface only when the incident evidence points there.

## Verification Checklist
- IdentityRecoveryKeyMassIssueInvestigationCritical and IdentityRecoveryKeyMassIssueInvestigationSloBurn are both resolved in Alertmanager for 30 minutes.
- oya_identity_recovery_key_mass_issue_investigation_error_ratio < 0.005 for 3 consecutive 10 minute windows.
- oya_identity_recovery_key_mass_issue_investigation_lag_seconds < 120 for all production cells.
- oya_identity_recovery_key_mass_issue_investigation_queue_depth is draining and not growing for the affected tenant.
- dashboard https://grafana.dev.oyatie.internal/d/identity-substrate/recovery-key-mass-issue-investigation?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=104 shows green panels for the affected cell.
- audit-chain query for EVT-IDENTITY-RECOVERY_KEY_MASS_ISSUE_INVESTIGATION-INCIDENT returns mitigation and resolution events.
- circuit breaker identity-recovery-key-mass-issue-investigation-circuit-breaker is closed after rollback window.
- feature flag oya.identity.recovery_key_mass_issue_investigation.incident_hold is false for the affected tenant unless long-term hold is approved.
- runbook invocation evidence is attached to evidence/incidents/$INCIDENT_ID.json.
- service owner acknowledged final handoff in #inc-identity-security.

## Postmortem Template
Use this exact skeleton for the incident document. The field names are intentionally stable for ADR-0263 audit emission extraction.
```markdown
---
doc_class: IncidentPostmortem
runbook_id: identity-recovery-key-mass-issue-investigation
microservice: identity
event_class: EVT-IDENTITY-RECOVERY_KEY_MASS_ISSUE_INVESTIGATION-INCIDENT
incident_id: <INC-...>
severity: sev0
status: draft
detected_at: <UTC>
mitigated_at: <UTC>
resolved_at: <UTC>
commander: <handle>
evidence_hash: <sha256>
---

# Recovery Key Mass Issue Investigation postmortem

## Summary
- What happened in identity/recovery-key-mass-issue-investigation.
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
- Emit EVT-IDENTITY-RECOVERY_KEY_MASS_ISSUE_INVESTIGATION-INCIDENT with incident_id, tenant_id, cell_id, principal_id, decision_id, evidence_hash, operator_id, runbook_id.
- Attach dashboard snapshot URLs and command transcripts.
- Seal mitigation and resolution events before closure.

## Corrective Actions
- Action, owner, due date, validation command, linked issue.
```

## Escalation Path
- Primary on-call: PagerDuty oya-identity-primary; ops-security secondary.
- Incident SLA: ack 3m for sev0/sev1, 10m for sev2, security checkpoint every 15m.
- Incident commander: first responder from axis-identity + ops-security; transfer only by explicit message in #inc-identity-security.
- Security escalation: page `ops-security-primary` immediately for sev0, data-boundary, credential, or audit-seal symptoms.
- Compliance escalation: page `dpo-office-duty` when tenant data, regulator evidence, or breach clock symptoms are present.
- Architecture escalation: page `council-architecture-reviewer` before manual bypass, policy rollback, or invariant relaxation.
- External vendors: Zitadel support; Yubico enterprise support; WebAuthn metadata service desk. Open a ticket once local dependency health is proven and vendor dependency remains suspect.
- Customer communications: use status page component `oyatie-identity-recovery-key-mass-issue-investigation` and keep private details in the incident channel.
- Regulatory clock: if any tenant data exposure is possible, start the compliance 72h assessment timer even if exposure is unconfirmed.
- Executive notice: sev0 or fleet-wide sev1 goes to `#exec-incident-readout` within 30 minutes.

## Cross-µservice Coordination
- Notify `tenancy`: `oya incident handoff --target tenancy --source identity --runbook recovery-key-mass-issue-investigation --incident $INCIDENT_ID --severity sev0 --branch A`; expect `202 accepted`.
- Require `tenancy` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `governance`: `oya incident handoff --target governance --source identity --runbook recovery-key-mass-issue-investigation --incident $INCIDENT_ID --severity sev0 --branch B`; expect `202 accepted`.
- Require `governance` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `compliance`: `oya incident handoff --target compliance --source identity --runbook recovery-key-mass-issue-investigation --incident $INCIDENT_ID --severity sev0 --branch C`; expect `202 accepted`.
- Require `compliance` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `observability`: `oya incident handoff --target observability --source identity --runbook recovery-key-mass-issue-investigation --incident $INCIDENT_ID --severity sev0 --branch D`; expect `202 accepted`.
- Require `observability` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Observability handoff API: `oya incident handoff --target observability --source identity --runbook recovery-key-mass-issue-investigation --incident $INCIDENT_ID`.
- Governance handoff API: `oya incident handoff --target governance --source identity --runbook recovery-key-mass-issue-investigation --incident $INCIDENT_ID`.
- Compliance handoff API: `oya incident handoff --target compliance --source identity --runbook recovery-key-mass-issue-investigation --incident $INCIDENT_ID`.
- Identity handoff API: `oya incident handoff --target identity --source identity --runbook recovery-key-mass-issue-investigation --incident $INCIDENT_ID`.
- Tenancy handoff API: `oya incident handoff --target tenancy --source identity --runbook recovery-key-mass-issue-investigation --incident $INCIDENT_ID`.

## Handoff Notes
- Do not hand off with only the alert name; include oya_identity_recovery_key_mass_issue_investigation_error_ratio, oya_identity_recovery_key_mass_issue_investigation_lag_seconds, oya_identity_recovery_key_mass_issue_investigation_queue_depth, current breaker state, and audit seal status.
- Keep identity-recovery-key-mass-issue-investigation-circuit-breaker owner as axis-identity + ops-security until the receiving service explicitly accepts.
- If another runbook owns the downstream fix, link this incident as upstream and keep this runbook open until downstream verification returns green.
- Close only after EVT-IDENTITY-RECOVERY_KEY_MASS_ISSUE_INVESTIGATION-INCIDENT has a sealed resolution row and every coordination endpoint above has either accepted or explicitly declined scope.

## Sources Checked During This Substance Pass
- `iam/identity/dashboards/` for dashboard names and operational panels.
- `iam/observability/slos/identity/` for OpenSLO alert vocabulary and threshold alignment.
- `iam/identity/policy/` for named policy and authorization surfaces.
- `iam/identity/catalog/` for component and owner vocabulary.
- Existing thin runbook topic `recovery-key-mass-issue-investigation` was preserved as the scenario anchor while replacing generic steps with concrete commands.
