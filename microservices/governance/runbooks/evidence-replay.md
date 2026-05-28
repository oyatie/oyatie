---
doc_class: Runbook
title: Evidence Replay
status: Accepted
date: 2026-05-20
microservice: governance
severity: sev0
audience: security-engineer
owner_team: axis-foundry + council-architecture + ops-sre-reliability
source_wave: codex-runbooks-substrate-w1
change_scope: substance rewrite of thin existing runbook
doc_status: published
---

# Runbook: Evidence Replay

## Operator Contract
- Runbook id: governance-evidence-replay.
- Primary service namespace: `governance`.
- Owning rotation: PagerDuty oya-governance-primary; council-architecture reviewer-on-call.
- Incident channel: `#inc-governance-gates`.
- External dependencies: Cedar policy runtime maintainers; Wasmtime security list; GitHub Enterprise support.
- API authority: `https://governance.internal.oyatie.dev/v1/governance/evidence-replay/incident-handoff`.
- Audit event class: `EVT-GOVERNANCE-EVIDENCE_REPLAY-INCIDENT` with ADR-0263 fields `incident_id`, `tenant_id`, `cell_id`, `microservice`, `runbook_id`, `decision_id`, `evidence_hash`, `operator_id`.
- Stop condition: mitigation has held for 30 minutes, GovernanceEvidenceReplayCritical is green, and all handoff APIs in Cross-µservice Coordination return `202 accepted`.
- Safety invariant: never clear the incident until `EVT-GOVERNANCE-EVIDENCE_REPLAY-INCIDENT` is sealed and the postmortem skeleton exists under `evidence/postmortems/governance-evidence-replay-<incident-id>.md`.

## Trigger Conditions
- Page on alert `GovernanceEvidenceReplayCritical` when `oya_governance_evidence_replay_error_ratio > 0.02` for 10 minutes in any production cell.
- Page on alert `GovernanceEvidenceReplaySloBurn` when `oya_governance_evidence_replay_lag_seconds > 300` for 2 consecutive evaluator windows.
- Open a sev0 if `oya_governance_evidence_replay_correctness_ratio < 0.9999` and the affected label set includes `tenant_id` or `principal_id`.
- Open a sev1 if `oya_governance_evidence_replay_queue_depth > 5000` for 15 minutes or retry backlog grows by more than 20 percent in one 5 minute window.
- Trigger from customer report when Support tags the case `governance.evidence-replay.customer_visible` in Zendesk.
- Trigger from CI when `cargo run -p oya-dev-cli -- gate validate governance-evidence-replay --production-snapshot` exits non-zero against the latest production evidence bundle.
- Primary dashboard: `https://grafana.dev.oyatie.internal/d/governance-substrate/evidence-replay?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=116`.
- Secondary dashboard: `https://grafana.dev.oyatie.internal/d/governance-substrate/evidence-replay?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=207`.
- Loki explorer: `https://grafana.dev.oyatie.internal/explore?query={namespace="governance",runbook="evidence-replay"}`.
- Alertmanager route: `oyatie-governance-evidence-replay-critical`; silence only with incident commander approval and `EVT-GOVERNANCE-EVIDENCE_REPLAY-INCIDENT` evidence.
- Synthetic probe: `oya ops probe governance evidence-replay --cell prod-us-east-1 --tenant synthetic-canary` returns `healthy=true`.
- Drift detector: `registry/governance/evidence-replay/expected-state.json` hash differs from live `https://governance.internal.oyatie.dev/v1/admin/state-hash`.

## Symptoms
- User-facing impact: evidence replay blocks or corrupts the governance control path for affected tenants.
- Operators see Grafana panel `lane-pass-rate / Evidence Replay burn rate` turn red before the primary alert resolves.
- Loki signature `governance.evidence_replay.incident_state=failed` appears with fields `incident_id`, `tenant_id`, `cell_id`, `decision_id`, `evidence_hash`.
- Kubernetes events include `reason=GovernanceEvidenceReplayDegraded` on deployment `governance-evidence-replay-worker`.
- Audit-chain shows missing or delayed `EVT-GOVERNANCE-EVIDENCE_REPLAY-INCIDENT` entries when queried with `oya audit-chain query --event-class EVT-GOVERNANCE-EVIDENCE_REPLAY-INCIDENT --since 30m`.
- Metric pattern: `oya_governance_evidence_replay_error_ratio` rises before `oya_governance_evidence_replay_lag_seconds`; if lag rises first, suspect dependency saturation rather than local regression.
- Metric pattern: `oya_governance_evidence_replay_queue_depth` increases while pod CPU stays below 40 percent; suspect downstream refusal or feature flag deadlock.
- Tenant-specific shape: one `tenant_id` dominates labels in `oya_governance_evidence_replay_queue_depth`; isolate before fleet mitigation.
- Fleet-wide shape: at least three cells report `GovernanceEvidenceReplayCritical` in one 15 minute window; switch to sev1 bridge even if individual tenants are low-volume.
- Log signature `decision=deny reason=evidence-replay.policy_guard` means the guard is working; investigate caller inputs before rollback.
- Log signature `decision=permit reason=evidence-replay.break_glass` means manual intervention is active; confirm two-person authorization.
- Log signature `audit_emit_status=stalled event_class=EVT-GOVERNANCE-EVIDENCE_REPLAY-INCIDENT` means mitigation cannot be closed until replay succeeds.

## Diagnostic Steps
1. Set incident variables: `export INCIDENT_ID=INC-governance-evidence-replay-$(date -u +%Y%m%dT%H%M%SZ); export CELL=prod-us-east-1; export TENANT=synthetic-canary`.
2. Confirm active alerts: `curl -s https://governance.internal.oyatie.dev/v1/alerts?runbook=evidence-replay | jq .alerts`.
3. Check Kubernetes rollout: `kubectl -n governance rollout status deploy/governance-evidence-replay-worker --timeout=60s`.
4. List unhealthy pods: `kubectl -n governance get pods -l app=governance-evidence-replay -o wide`.
5. Read structured logs: `kubectl -n governance logs deploy/governance-evidence-replay-worker --since=30m | rg "governance.evidence_replay.incident_state|GovernanceEvidenceReplayCritical|EVT-GOVERNANCE-EVIDENCE_REPLAY-INCIDENT"`.
6. Query Loki directly: `logcli query '{namespace="governance",runbook="evidence-replay"}' --since=30m --limit=200`.
7. Check Prometheus fast burn: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_governance_evidence_replay_error_ratio{cell="prod-us-east-1"}'`.
8. Check lag: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_governance_evidence_replay_lag_seconds{cell="prod-us-east-1"}'`.
9. Check queue: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_governance_evidence_replay_queue_depth{cell="prod-us-east-1"}'`.
10. Open primary dashboard: `open "https://grafana.dev.oyatie.internal/d/governance-substrate/evidence-replay?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=116&var-incident=$INCIDENT_ID"`.
11. Open secondary dashboard: `open "https://grafana.dev.oyatie.internal/d/governance-substrate/evidence-replay?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=207&var-tenant=$TENANT"`.
12. Verify audit-chain emission: `oya audit-chain query --event-class EVT-GOVERNANCE-EVIDENCE_REPLAY-INCIDENT --since 30m --cell $CELL --tenant $TENANT`.
13. Verify service state: `oya ops governance evidence-replay status --cell $CELL --tenant $TENANT --output json`.
14. Run production snapshot gate: `cargo run -p oya-dev-cli -- gate validate governance-evidence-replay --production-snapshot --cell $CELL`.
15. Check Cargo owner crate: `cargo test -p oya-governance-domain evidence_replay -- --nocapture`.
16. Check API contract smoke: `curl -s https://governance.internal.oyatie.dev/v1/governance/evidence-replay/incident-handoff -H "x-oya-tenant: $TENANT"`.
17. Inspect config: `kubectl -n governance get configmap governance-evidence-replay-config -o yaml`.
18. Inspect feature flags: `oya flags get oya.governance.evidence_replay.incident_hold --cell $CELL --tenant $TENANT --output yaml`.
19. Inspect circuit breaker: `oya ops breaker status governance-evidence-replay-circuit-breaker --cell $CELL --tenant $TENANT`.
20. Check recent deploy: `kubectl -n governance rollout history deploy/governance-evidence-replay-worker | tail -20`.
21. Check policy file: `test -f microservices/governance/policy/lane-execution.cedar || test -f microservices/governance/policy/lane-execution.md`.
22. Check SLO files: `ls microservices/governance/slos/*.openslo.yaml | sort`.
23. Check catalog components: `find microservices/governance/catalog -maxdepth 1 -type f | sort | rg "governance|evidence"`.
24. Confirm no cross-cell spread: `oya ops cells query --metric oya_governance_evidence_replay_error_ratio --window 30m --threshold 0.02`.
25. Snapshot evidence: `oya evidence snapshot --incident $INCIDENT_ID --microservice governance --runbook evidence-replay --output evidence/incidents/$INCIDENT_ID.json`.

### Diagnostic Decision Tree
```text
Evidence Replay incident decision tree
1. Is GovernanceEvidenceReplayCritical firing in more than one cell?
   |-- yes: declare fleet incident, page PagerDuty oya-governance-primary; council-architecture reviewer-on-call, and run cross-cell containment.
   |-- no: keep scope to the affected cell and continue tenant isolation checks.
2. Does oya_governance_evidence_replay_queue_depth grow while oya_governance_evidence_replay_error_ratio is flat?
   |-- yes: downstream dependency or replay backlog; choose mitigation branch B.
   |-- no: local regression or bad input; continue branch selection.
3. Does audit-chain show EVT-GOVERNANCE-EVIDENCE_REPLAY-INCIDENT gaps?
   |-- yes: do not close; run evidence replay before resolution.
   |-- no: mitigation can proceed after state is green.
4. Is customer or regulator impact confirmed?
   |-- yes: promote severity, open #inc-governance-gates, and notify compliance handoff.
   |-- no: keep internal incident and collect evidence.
```
- Branch A (confirmed security boundary failure): use the matching mitigation block below and record `decision_branch=A` in `EVT-GOVERNANCE-EVIDENCE_REPLAY-INCIDENT`.
- Branch B (suspected false positive): use the matching mitigation block below and record `decision_branch=B` in `EVT-GOVERNANCE-EVIDENCE_REPLAY-INCIDENT`.
- Branch C (forensic evidence unavailable): use the matching mitigation block below and record `decision_branch=C` in `EVT-GOVERNANCE-EVIDENCE_REPLAY-INCIDENT`.
- Branch D (customer or regulator visible impact): use the matching mitigation block below and record `decision_branch=D` in `EVT-GOVERNANCE-EVIDENCE_REPLAY-INCIDENT`.

## Mitigation Steps
1. Acknowledge page: `pd incident ack --service governance --incident $INCIDENT_ID`.
2. Create bridge: `oya incident bridge create --incident $INCIDENT_ID --channel #inc-governance-gates --severity sev0`.
3. Freeze risky automation: `oya flags set oya.governance.evidence_replay.incident_hold=true --cell $CELL --tenant $TENANT --reason $INCIDENT_ID`.
4. Enable circuit breaker: `oya ops breaker open governance-evidence-replay-circuit-breaker --cell $CELL --tenant $TENANT --ttl 30m --reason $INCIDENT_ID`.
5. Reduce blast radius: `kubectl -n governance scale deploy/governance-evidence-replay-worker --replicas=1`.
6. Protect tenant boundary: `oya tenancy quarantine --tenant $TENANT --reason governance-evidence-replay --ttl 60m`.
7. Pause promotion: incident hold PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
8. Drain queue safely: `oya ops governance evidence-replay drain --cell $CELL --tenant $TENANT --max-items 500 --dry-run`.
9. Execute bounded drain: `oya ops governance evidence-replay drain --cell $CELL --tenant $TENANT --max-items 500 --confirm $INCIDENT_ID`.
10. Replay missing audit events: `oya audit-chain replay --event-class EVT-GOVERNANCE-EVIDENCE_REPLAY-INCIDENT --incident $INCIDENT_ID --from evidence/incidents/$INCIDENT_ID.json`.
11. Rollback last deploy if causal: `kubectl -n governance rollout undo deploy/governance-evidence-replay-worker`.
12. Raise HPA cap if saturation: `kubectl -n governance patch hpa governance-evidence-replay-worker --type merge -p '{"spec":{"maxReplicas":12}}'`.
13. Throttle hot tenant: `oya ops rate-limit set --tenant $TENANT --surface governance.evidence-replay --rps 25 --ttl 30m`.
14. Block abusive principal: `oya identity principal suspend --principal suspected-abuse --tenant $TENANT --reason $INCIDENT_ID`.
15. Protect evidence: `oya evidence freeze --incident $INCIDENT_ID --paths microservices/governance/runbooks/evidence-replay.md,evidence/incidents/$INCIDENT_ID.json`.
16. Notify service owners: `oya notify service-owner --microservice governance --incident $INCIDENT_ID --channel #inc-governance-gates`.
17. Open external vendor ticket: `oya vendor ticket open --vendor primary-governance --incident $INCIDENT_ID --summary evidence-replay`.
18. Confirm breaker effect: `oya ops breaker status governance-evidence-replay-circuit-breaker --cell $CELL --tenant $TENANT --expect open`.
19. Confirm user impact reduced: `curl -s https://governance.internal.oyatie.dev/v1/governance/evidence-replay/incident-handoff/health -H "x-oya-tenant: $TENANT"`.
20. Emit mitigation audit: `oya audit-chain emit --event-class EVT-GOVERNANCE-EVIDENCE_REPLAY-INCIDENT --incident $INCIDENT_ID --field mitigation=active --field runbook=evidence-replay`.

### Mitigation Branch Guidance
- Branch A: confirmed security boundary failure.
  - Required action: keep `governance-evidence-replay-circuit-breaker` open until `oya_governance_evidence_replay_error_ratio` is below 0.005 for 3 windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/governance-substrate/evidence-replay?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=116` to the incident.
  - Required audit: emit `EVT-GOVERNANCE-EVIDENCE_REPLAY-INCIDENT` with `branch=A`, `operator_id`, and `evidence_hash`.
- Branch B: suspected false positive.
  - Required action: keep `governance-evidence-replay-circuit-breaker` open until `oya_governance_evidence_replay_error_ratio` is below 0.005 for 3 windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/governance-substrate/evidence-replay?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=117` to the incident.
  - Required audit: emit `EVT-GOVERNANCE-EVIDENCE_REPLAY-INCIDENT` with `branch=B`, `operator_id`, and `evidence_hash`.
- Branch C: forensic evidence unavailable.
  - Required action: keep `governance-evidence-replay-circuit-breaker` open until `oya_governance_evidence_replay_error_ratio` is below 0.005 for 3 windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/governance-substrate/evidence-replay?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=118` to the incident.
  - Required audit: emit `EVT-GOVERNANCE-EVIDENCE_REPLAY-INCIDENT` with `branch=C`, `operator_id`, and `evidence_hash`.
- Branch D: customer or regulator visible impact.
  - Required action: keep `governance-evidence-replay-circuit-breaker` open until `oya_governance_evidence_replay_error_ratio` is below 0.005 for 3 windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/governance-substrate/evidence-replay?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=119` to the incident.
  - Required audit: emit `EVT-GOVERNANCE-EVIDENCE_REPLAY-INCIDENT` with `branch=D`, `operator_id`, and `evidence_hash`.

## Resolution Steps
1. Identify code owner path: `rg "evidence_replay|GovernanceEvidenceReplayCritical|governance.evidence_replay.incident_state" crates microservices/governance -g "!microservices/governance/runbooks/**"`.
2. Patch domain invariant: `edit oya-governance-domain where evidence_replay state transition is validated`.
3. Patch API guard: `edit microservices/governance/contracts/openapi.yaml or catalog REST binding if the failing path is north-south`.
4. Patch policy: `edit microservices/governance/policy/lane-execution.cedar or .md with explicit deny/permit branch`.
5. Patch runtime config: `edit microservices/governance/iac/k8s-deployment.yaml or secret-bindings.yaml if deploy/config drift caused the incident`.
6. Add regression test: `cargo test -p oya-governance-domain evidence_replay_incident_regression -- --nocapture`.
7. Add gate evidence: `cargo run -p oya-dev-cli -- gate validate governance-evidence-replay --fixture incident-evidence-replay.json`.
8. Add SLO assertion: `update microservices/governance/slos/* with alert GovernanceEvidenceReplayCritical when this was a missing alert`.
9. Add dashboard panel: `update microservices/governance/dashboards/lane-pass-rate.json with oya_governance_evidence_replay_error_ratio, oya_governance_evidence_replay_lag_seconds, and oya_governance_evidence_replay_queue_depth`.
10. Rebuild affected crate: `cargo check -p oya-governance-domain --all-targets`.
11. Run targeted tests: `cargo test -p oya-governance-domain --all-features`.
12. Run policy validation: `cargo run -p oya-dev-cli -- gate validate governance-policy --microservice governance`.
13. Deploy canary: `oya deploy canary --microservice governance --component evidence-replay-worker --cell $CELL --weight 1`.
14. Watch burn rate: `oya ops watch --metric oya_governance_evidence_replay_error_ratio --threshold 0.005 --window 30m --cell $CELL`.
15. Close circuit breaker: `oya ops breaker close governance-evidence-replay-circuit-breaker --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
16. Unfreeze automation: `oya flags set oya.governance.evidence_replay.incident_hold=false --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
17. Resume promotion: recovery PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
18. Seal resolution audit: `oya audit-chain emit --event-class EVT-GOVERNANCE-EVIDENCE_REPLAY-INCIDENT --incident $INCIDENT_ID --field resolution=complete --field runbook=evidence-replay`.
19. Verify seal: `oya audit-chain verify --event-class EVT-GOVERNANCE-EVIDENCE_REPLAY-INCIDENT --incident $INCIDENT_ID`.
20. Attach final evidence: `oya evidence attach --incident $INCIDENT_ID --file evidence/incidents/$INCIDENT_ID.json --kind final-resolution`.

### Code Paths To Inspect First
- `oya-governance-domain`: inspect for evidence_replay invariants, alert emission, and ADR-0263 evidence fields before touching adjacent code path 1.
- `oya-dev-cli`: inspect for evidence_replay invariants, alert emission, and ADR-0263 evidence fields before touching adjacent code path 2.
- `oya-policy-cedar-domain`: inspect for evidence_replay invariants, alert emission, and ADR-0263 evidence fields before touching adjacent code path 3.
- `microservices/governance/contracts/`: verify this surface only when the incident evidence points there.
- `microservices/governance/dashboards/lane-pass-rate.json`: verify this surface only when the incident evidence points there.
- `microservices/governance/slos/`: verify this surface only when the incident evidence points there.
- `microservices/governance/policy/lane-execution.*`: verify this surface only when the incident evidence points there.

## Verification Checklist
- GovernanceEvidenceReplayCritical and GovernanceEvidenceReplaySloBurn are both resolved in Alertmanager for 30 minutes.
- oya_governance_evidence_replay_error_ratio < 0.005 for 3 consecutive 10 minute windows.
- oya_governance_evidence_replay_lag_seconds < 120 for all production cells.
- oya_governance_evidence_replay_queue_depth is draining and not growing for the affected tenant.
- dashboard https://grafana.dev.oyatie.internal/d/governance-substrate/evidence-replay?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=116 shows green panels for the affected cell.
- audit-chain query for EVT-GOVERNANCE-EVIDENCE_REPLAY-INCIDENT returns mitigation and resolution events.
- circuit breaker governance-evidence-replay-circuit-breaker is closed after rollback window.
- feature flag oya.governance.evidence_replay.incident_hold is false for the affected tenant unless long-term hold is approved.
- runbook invocation evidence is attached to evidence/incidents/$INCIDENT_ID.json.
- service owner acknowledged final handoff in #inc-governance-gates.

## Postmortem Template
Use this exact skeleton for the incident document. The field names are intentionally stable for ADR-0263 audit emission extraction.
```markdown
---
doc_class: IncidentPostmortem
runbook_id: governance-evidence-replay
microservice: governance
event_class: EVT-GOVERNANCE-EVIDENCE_REPLAY-INCIDENT
incident_id: <INC-...>
severity: sev0
status: draft
detected_at: <UTC>
mitigated_at: <UTC>
resolved_at: <UTC>
commander: <handle>
evidence_hash: <sha256>
---

# Evidence Replay postmortem

## Summary
- What happened in governance/evidence-replay.
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
- Emit EVT-GOVERNANCE-EVIDENCE_REPLAY-INCIDENT with incident_id, tenant_id, cell_id, principal_id, decision_id, evidence_hash, operator_id, runbook_id.
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
- Customer communications: use status page component `oyatie-governance-evidence-replay` and keep private details in the incident channel.
- Regulatory clock: if any tenant data exposure is possible, start the compliance 72h assessment timer even if exposure is unconfirmed.
- Executive notice: sev0 or fleet-wide sev1 goes to `#exec-incident-readout` within 30 minutes.

## Cross-µservice Coordination
- Notify `identity`: `oya incident handoff --target identity --source governance --runbook evidence-replay --incident $INCIDENT_ID --severity sev0 --branch A`; expect `202 accepted`.
- Require `identity` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `tenancy`: `oya incident handoff --target tenancy --source governance --runbook evidence-replay --incident $INCIDENT_ID --severity sev0 --branch B`; expect `202 accepted`.
- Require `tenancy` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `compliance`: `oya incident handoff --target compliance --source governance --runbook evidence-replay --incident $INCIDENT_ID --severity sev0 --branch C`; expect `202 accepted`.
- Require `compliance` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `observability`: `oya incident handoff --target observability --source governance --runbook evidence-replay --incident $INCIDENT_ID --severity sev0 --branch D`; expect `202 accepted`.
- Require `observability` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Observability handoff API: `oya incident handoff --target observability --source governance --runbook evidence-replay --incident $INCIDENT_ID`.
- Governance handoff API: `oya incident handoff --target governance --source governance --runbook evidence-replay --incident $INCIDENT_ID`.
- Compliance handoff API: `oya incident handoff --target compliance --source governance --runbook evidence-replay --incident $INCIDENT_ID`.
- Identity handoff API: `oya incident handoff --target identity --source governance --runbook evidence-replay --incident $INCIDENT_ID`.
- Tenancy handoff API: `oya incident handoff --target tenancy --source governance --runbook evidence-replay --incident $INCIDENT_ID`.

## Handoff Notes
- Do not hand off with only the alert name; include oya_governance_evidence_replay_error_ratio, oya_governance_evidence_replay_lag_seconds, oya_governance_evidence_replay_queue_depth, current breaker state, and audit seal status.
- Keep governance-evidence-replay-circuit-breaker owner as axis-foundry + council-architecture + ops-sre-reliability until the receiving service explicitly accepts.
- If another runbook owns the downstream fix, link this incident as upstream and keep this runbook open until downstream verification returns green.
- Close only after EVT-GOVERNANCE-EVIDENCE_REPLAY-INCIDENT has a sealed resolution row and every coordination endpoint above has either accepted or explicitly declined scope.

## Sources Checked During This Substance Pass
- `microservices/governance/dashboards/` for dashboard names and operational panels.
- `microservices/governance/slos/` for OpenSLO alert vocabulary and threshold alignment.
- `microservices/governance/policy/` for named policy and authorization surfaces.
- `microservices/governance/catalog/` for component and owner vocabulary.
- Existing thin runbook topic `evidence-replay` was preserved as the scenario anchor while replacing generic steps with concrete commands.
