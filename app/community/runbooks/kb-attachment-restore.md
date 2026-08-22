---
doc_class: Runbook
title: Kb Attachment Restore
status: Accepted
date: 2026-05-20
microservice: community
severity: sev2
audience: community-trust-and-safety-on-call
owner_team: axis-community + ops-sre-reliability
source_wave: codex-runbooks-substrate-w3
change_scope: substance rewrite of existing thin runbook
doc_status: published
---

# Runbook: Kb Attachment Restore

## Operator Contract
- Runbook id: community-kb-attachment-restore.
- Primary service namespace: `community`.
- Owning rotation: PagerDuty community-primary; trust-safety-secondary.
- Incident channel: `#inc-community`.
- Operational focus: knowledge-base attachment must be restored without breaking tenant data residency.
- Named precedent: this follows the Reddit moderation queue plus Stack Overflow vote-integrity and Cloudflare abuse-response pattern.
- External dependencies: Cloudflare Trust and Safety; OpenSearch support; Zendesk Trust Center support.
- API authority: `https://community.internal.oyatie.dev/v1/community/kb-attachment-restore/incident-handoff`.
- Audit event class: `EVT_COMMUNITY_KB_ATTACHMENT_RESTORE_INCIDENT` with ADR-0263 fields `incident_id`, `tenant_id`, `cell_id`, `microservice`, `runbook_id`, `decision_id`, `evidence_hash`, `operator_id`.
- Stop condition: mitigation has held for 30 minutes, `KbAttachmentRestoreCritical` is green, and every Cross-microservice handoff API returns `202 accepted`.
- Safety invariant: never clear the incident until `EVT_COMMUNITY_KB_ATTACHMENT_RESTORE_INCIDENT` is sealed and the postmortem skeleton exists under `evidence/postmortems/community-kb-attachment-restore-<incident-id>.md`.

## Trigger Conditions
- Page on alert `KbAttachmentRestoreCritical` when `community_kb_attachment_restore_error_ratio > 0.02` for 10 minutes in any production cell.
- Page on alert `KbAttachmentRestoreSloBurn` when `community_kb_attachment_restore_lag_seconds > 300` for 2 consecutive evaluator windows.
- Open sev2 if `community_kb_attachment_restore_total` exceeds the threshold documented in `app/community/slos/audit-chain-seal-latency.openslo.yaml`.
- Open sev2 if `community_kb_attachment_restore_queue_depth > 5000` for 15 minutes or retry backlog grows by more than 20 percent in one 5 minute window.
- Trigger from customer report when Support tags the case `community.kb-attachment-restore.customer_visible` in Zendesk.
- Trigger from CI when `cargo run -p dev-cli -- gate validate community-kb-attachment-restore --production-snapshot` exits non-zero against the latest production evidence bundle.
- Primary dashboard: `https://grafana.dev.oyatie.internal/d/community-ops/kb-attachment-restore?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=101` backed by `app/community/dashboards/moderation-queue-depth.json`.
- Secondary dashboard: `https://grafana.dev.oyatie.internal/d/community-ops/kb-attachment-restore?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=202` backed by `app/community/dashboards/post-throughput.json`.
- Loki explorer: `https://grafana.dev.oyatie.internal/explore?query={namespace="community",runbook="kb-attachment-restore"}`.
- Alertmanager route: `oyatie-community-kb-attachment-restore-critical`; silence only with incident commander approval and `EVT_COMMUNITY_KB_ATTACHMENT_RESTORE_INCIDENT` evidence.
- Synthetic probe: `oya ops probe community kb-attachment-restore --cell prod-us-east-1 --tenant synthetic-canary` returns `healthy=true`.
- Drift detector: `registry/community/kb-attachment-restore/expected-state.json` hash differs from live `https://community.internal.oyatie.dev/v1/community/kb-attachment-restore/admin/state-hash`.
- Service-specific metric `community_kb_attachment_restore_total` is red while `community_kb_attachment_restore_audit_emit_total{status="sealed"}` is flat.

## Symptoms
- User-facing impact: members, moderators, or tenant admins may see moderation, anonymity, search, voting, or post integrity failures; scenario focus is knowledge-base attachment must be restored without breaking tenant data residency.
- Operators see Grafana panel `moderation-queue-depth.json / Kb Attachment Restore burn rate` turn red before the primary alert resolves.
- Loki signature `community.kb_attachment_restore.incident_state=failed` appears with fields `incident_id`, `tenant_id`, `cell_id`, `decision_id`, `evidence_hash`.
- Kubernetes events include `reason=KbAttachmentRestoreDegraded` on deployment `community-kb-attachment-restore-worker` or `community-api`.
- Audit-chain shows missing or delayed `EVT_COMMUNITY_KB_ATTACHMENT_RESTORE_INCIDENT` entries when queried with `oya audit-chain query --event-class EVT_COMMUNITY_KB_ATTACHMENT_RESTORE_INCIDENT --since 30m`.
- Metric pattern: `community_kb_attachment_restore_error_ratio` rises before `community_kb_attachment_restore_lag_seconds`; if lag rises first, suspect dependency saturation rather than local regression.
- Metric pattern: `community_kb_attachment_restore_queue_depth` increases while pod CPU stays below 40 percent; suspect downstream refusal, replay backlog, or feature flag deadlock.
- Tenant-specific shape: one `tenant_id` dominates labels in `community_kb_attachment_restore_queue_depth`; isolate tenant before fleet mitigation.
- Fleet-wide shape: at least three cells report `KbAttachmentRestoreCritical` in one 15 minute window; switch to cross-cell bridge even if individual tenants are low-volume.
- Log signature `decision=deny reason=kb-attachment-restore.policy_guard` means the guard is working; investigate caller inputs before rollback.
- Log signature `decision=permit reason=kb-attachment-restore.break_glass` means manual intervention is active; confirm two-person authorization.
- Log signature `audit_emit_status=stalled event_class=EVT_COMMUNITY_KB_ATTACHMENT_RESTORE_INCIDENT` means mitigation cannot be closed until replay succeeds.
- Service-specific pattern: `community_kb_attachment_restore_total` rises while `community_kb_attachment_restore_dependency_error_ratio` is flat; inspect local state before escalating Cloudflare Trust and Safety.
- Service-specific pattern: `community_kb_attachment_restore_dependency_error_ratio` rises while `community_kb_attachment_restore_total` is flat; inspect vendor or adjacent-service dependency health before local rollback.

## Failure Mode Tree
- Failure mode 1: single-tenant CommunityThread inconsistency; contain with tenant quarantine, preserve all `EVT_COMMUNITY_KB_ATTACHMENT_RESTORE_INCIDENT` rows, and avoid fleet rollback.
- Failure mode 2: cross-cell ModerationDecision drift; freeze writes, compare state hash across cells, and use audit-chain replay before accepting new mutations.
- Failure mode 3: byzantine or abusive principal; suspend the principal through identity, keep tenant data scoped, and preserve Cedar explain output.
- Failure mode 4: external dependency outage at Cloudflare Trust and Safety; open vendor ticket only after local dashboards and handoff APIs prove the dependency is causal.
- Failure mode 5: operator mitigation made state worse; roll back feature flag `oya.community.kb_attachment_restore.incident_hold`, close `community-kb-attachment-restore-circuit-breaker`, and restore the previous deployment revision.
- Failure mode 6: audit emission is delayed; do not close even when customer symptoms improve because ADR-0263 evidence is incomplete.
- Failure mode 7: regional partition; keep prod-us-east-1 as evidence leader and reject cross-region mutation until `community_kb_attachment_restore_state_hash_match == 1`.
- Failure mode 8: compliance-pack mismatch; require compliance handoff when KR-CSAP, EU-sovereign, FedRAMP-High, IL5, or CN-PIPL labels are present.
- Failure mode 9: stale dashboard data; verify direct Mimir queries before making rollback decisions.
- Failure mode 10: runbook step ambiguity; halt the ambiguous branch, emit `EVT_COMMUNITY_KB_ATTACHMENT_RESTORE_INCIDENT` with outcome `blocked`, and patch this runbook after recovery.

## Diagnostic Steps
1. Set incident variables: `export INCIDENT_ID=INC-community-kb-attachment-restore-$(date -u +%Y%m%dT%H%M%SZ); export CELL=prod-us-east-1; export TENANT=synthetic-canary`.
2. Confirm active alerts: `curl -s https://community.internal.oyatie.dev/v1/community/alerts?runbook=kb-attachment-restore | jq .alerts`.
3. Check Kubernetes rollout: `kubectl -n community rollout status deploy/community-kb-attachment-restore-worker --timeout=60s`.
4. List unhealthy pods: `kubectl -n community get pods -l app=kb-attachment-restore -o wide`.
5. Read structured logs: `kubectl -n community logs deploy/community-kb-attachment-restore-worker --since=30m | rg "community.kb_attachment_restore.incident_state|KbAttachmentRestoreCritical|EVT_COMMUNITY_KB_ATTACHMENT_RESTORE_INCIDENT"`.
6. Query Loki directly: `logcli query '{namespace="community",runbook="kb-attachment-restore"}' --since=30m --limit=200`.
7. Check Prometheus error ratio: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=community_kb_attachment_restore_error_ratio{cell="prod-us-east-1"}'`.
8. Check lag: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=community_kb_attachment_restore_lag_seconds{cell="prod-us-east-1"}'`.
9. Check queue: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=community_kb_attachment_restore_queue_depth{cell="prod-us-east-1"}'`.
10. Check service-specific signal: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=community_kb_attachment_restore_total{cell="prod-us-east-1"}'`.
11. Open primary dashboard: `open "https://grafana.dev.oyatie.internal/d/community-ops/kb-attachment-restore?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=101&var-incident=$INCIDENT_ID"`.
12. Open secondary dashboard: `open "https://grafana.dev.oyatie.internal/d/community-ops/kb-attachment-restore?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=202&var-tenant=$TENANT"`.
13. Verify audit-chain emission: `oya audit-chain query --event-class EVT_COMMUNITY_KB_ATTACHMENT_RESTORE_INCIDENT --since 30m --cell $CELL --tenant $TENANT`.
14. Verify service state: `oya ops community kb-attachment-restore status --cell $CELL --tenant $TENANT --output json`.
15. Run production snapshot gate: `cargo run -p dev-cli -- gate validate community-kb-attachment-restore --production-snapshot --cell $CELL`.
16. Run crate smoke test: `cargo test -p community-moderation-queue-domain kb_attachment_restore -- --nocapture`.
17. Check API contract smoke: `curl -s https://community.internal.oyatie.dev/v1/community/kb-attachment-restore/incident-handoff -H "x-tenant: $TENANT"`.
18. Inspect config: `test -f app/community/iac/kustomize/base/kustomization.yaml && sed -n '1,180p' app/community/iac/kustomize/base/kustomization.yaml`.
19. Inspect feature flags: `oya flags get oya.community.kb_attachment_restore.incident_hold --cell $CELL --tenant $TENANT --output yaml`.
20. Inspect circuit breaker: `oya ops breaker status community-kb-attachment-restore-circuit-breaker --cell $CELL --tenant $TENANT`.
21. Check recent deploy: `kubectl -n community rollout history deploy/community-kb-attachment-restore-worker | tail -20`.
22. Check policy file: `test -f app/community/policy/community-isolation.md || find app/community/policy -maxdepth 2 -type f | sort`.
23. Check SLO files: `ls app/community/slos/*.openslo.yaml | sort | rg "audit|feed"`.
24. Check contract binding: `test -f app/community/contracts/openapi/community.yaml && sed -n '1,120p' app/community/contracts/openapi/community.yaml`.
25. Run targeted SQL state query: `psql $OYATIE_PROD_DSN -c "select incident_id, tenant_id, cell_id, state, updated_at from community_kb_attachment_restore_incidents where updated_at > now() - interval '30 minutes' order by updated_at desc limit 20;"`.
26. Confirm no cross-cell spread: `oya ops cells query --metric community_kb_attachment_restore_error_ratio --window 30m --threshold 0.02`.
27. Snapshot evidence: `oya evidence snapshot --incident $INCIDENT_ID --microservice community --runbook kb-attachment-restore --output evidence/incidents/$INCIDENT_ID.json`.

### Diagnostic Decision Tree
```text
Kb Attachment Restore incident decision tree
1. Is KbAttachmentRestoreCritical firing in more than one cell?
   |-- yes: declare fleet incident, page PagerDuty community-primary, and run cross-cell containment.
   |-- no: keep scope to the affected cell and continue tenant isolation checks.
2. Does community_kb_attachment_restore_queue_depth grow while community_kb_attachment_restore_error_ratio is flat?
   |-- yes: downstream dependency, replay backlog, or queue-drain issue; choose mitigation branch B.
   |-- no: local regression, bad input, or policy/config drift; continue branch selection.
3. Does audit-chain show EVT_COMMUNITY_KB_ATTACHMENT_RESTORE_INCIDENT gaps?
   |-- yes: do not close; run evidence replay before resolution.
   |-- no: mitigation can proceed after state is green.
4. Is customer, finance, security, or regulator impact confirmed?
   |-- yes: promote severity, open #inc-community, and notify compliance or security handoff.
   |-- no: keep internal incident and collect evidence.
```
- Branch A (confirmed CommunityThread correctness risk): use the matching mitigation block below and record `decision_branch=A` in `EVT_COMMUNITY_KB_ATTACHMENT_RESTORE_INCIDENT`.
- Branch B (dependency saturation or replay backlog): use the matching mitigation block below and record `decision_branch=B` in `EVT_COMMUNITY_KB_ATTACHMENT_RESTORE_INCIDENT`.
- Branch C (policy, permit, or tenant-scope drift): use the matching mitigation block below and record `decision_branch=C` in `EVT_COMMUNITY_KB_ATTACHMENT_RESTORE_INCIDENT`.
- Branch D (customer-visible or regulated evidence gap): use the matching mitigation block below and record `decision_branch=D` in `EVT_COMMUNITY_KB_ATTACHMENT_RESTORE_INCIDENT`.

## Mitigation Steps
1. Acknowledge page: `pd incident ack --service community --incident $INCIDENT_ID`.
2. Create bridge: `oya incident bridge create --incident $INCIDENT_ID --channel #inc-community --severity sev2`.
3. Freeze risky automation: `oya flags set oya.community.kb_attachment_restore.incident_hold=true --cell $CELL --tenant $TENANT --reason $INCIDENT_ID`.
4. Enable circuit breaker: `oya ops breaker open community-kb-attachment-restore-circuit-breaker --cell $CELL --tenant $TENANT --ttl 30m --reason $INCIDENT_ID`.
5. Reduce blast radius: `kubectl -n community scale deploy/community-kb-attachment-restore-worker --replicas=1`.
6. Protect tenant boundary: `oya tenancy quarantine --tenant $TENANT --reason community-kb-attachment-restore --ttl 60m`.
7. Pause promotion: incident hold PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
8. Drain queue safely: `oya ops community kb-attachment-restore drain --cell $CELL --tenant $TENANT --max-items 500 --dry-run`.
9. Execute bounded drain: `oya ops community kb-attachment-restore drain --cell $CELL --tenant $TENANT --max-items 500 --confirm $INCIDENT_ID`.
10. Replay missing audit events: `oya audit-chain replay --event-class EVT_COMMUNITY_KB_ATTACHMENT_RESTORE_INCIDENT --incident $INCIDENT_ID --from evidence/incidents/$INCIDENT_ID.json`.
11. Rollback last deploy if causal: `kubectl -n community rollout undo deploy/community-kb-attachment-restore-worker`.
12. Raise HPA cap if saturation is proven: `kubectl -n community patch hpa community-kb-attachment-restore-worker --type merge -p '{"spec":{"maxReplicas":12}}'`.
13. Throttle hot tenant: `oya ops rate-limit set --tenant $TENANT --surface community.kb-attachment-restore --rps 25 --ttl 30m`.
14. Block abusive principal when relevant: `oya identity principal suspend --principal suspected-abuse --tenant $TENANT --reason $INCIDENT_ID`.
15. Protect evidence: `oya evidence freeze --incident $INCIDENT_ID --paths app/community/runbooks/kb-attachment-restore.md,evidence/incidents/$INCIDENT_ID.json`.
16. Notify service owners: `oya notify service-owner --microservice community --incident $INCIDENT_ID --channel #inc-community`.
17. Open external vendor ticket: `oya vendor ticket open --vendor "Cloudflare Trust and Safety" --incident $INCIDENT_ID --summary community-kb-attachment-restore`.
18. Confirm breaker effect: `oya ops breaker status community-kb-attachment-restore-circuit-breaker --cell $CELL --tenant $TENANT --expect open`.
19. Confirm user impact reduced: `curl -s https://community.internal.oyatie.dev/v1/community/kb-attachment-restore/incident-handoff/health -H "x-tenant: $TENANT"`.
20. Emit mitigation audit: `oya audit-chain emit --event-class EVT_COMMUNITY_KB_ATTACHMENT_RESTORE_INCIDENT --incident $INCIDENT_ID --field mitigation=active --field runbook=kb-attachment-restore`.

### Mitigation Branch Guidance
- Branch A: confirmed CommunityThread correctness risk.
  - Required action: keep `community-kb-attachment-restore-circuit-breaker` open until `community_kb_attachment_restore_error_ratio` is below 0.005 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/community-ops/kb-attachment-restore?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=110` to the incident.
  - Required audit: emit `EVT_COMMUNITY_KB_ATTACHMENT_RESTORE_INCIDENT` with `branch=A`, `operator_id`, and `evidence_hash`.
- Branch B: dependency saturation or replay backlog.
  - Required action: keep `community-kb-attachment-restore-circuit-breaker` open until `community_kb_attachment_restore_error_ratio` is below 0.01 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/community-ops/kb-attachment-restore?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=111` to the incident.
  - Required audit: emit `EVT_COMMUNITY_KB_ATTACHMENT_RESTORE_INCIDENT` with `branch=B`, `operator_id`, and `evidence_hash`.
- Branch C: policy, permit, or tenant-scope drift.
  - Required action: keep `community-kb-attachment-restore-circuit-breaker` open until `community_kb_attachment_restore_error_ratio` is below 0.005 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/community-ops/kb-attachment-restore?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=112` to the incident.
  - Required audit: emit `EVT_COMMUNITY_KB_ATTACHMENT_RESTORE_INCIDENT` with `branch=C`, `operator_id`, and `evidence_hash`.
- Branch D: customer-visible or regulated evidence gap.
  - Required action: keep `community-kb-attachment-restore-circuit-breaker` open until `community_kb_attachment_restore_error_ratio` is below 0.01 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/community-ops/kb-attachment-restore?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=113` to the incident.
  - Required audit: emit `EVT_COMMUNITY_KB_ATTACHMENT_RESTORE_INCIDENT` with `branch=D`, `operator_id`, and `evidence_hash`.

## Resolution Steps
1. Identify code owner path: `rg "kb_attachment_restore|KbAttachmentRestoreCritical|community.kb_attachment_restore.incident_state" crates app/community -g "!app/community/runbooks/**"`.
2. Patch domain invariant: `edit community-moderation-queue-domain where kb_attachment_restore state transition is validated`.
3. Patch API guard: `edit app/community/contracts/openapi/community.yaml if the failing path is north-south or async handoff`.
4. Patch policy: `edit app/community/policy/community-isolation.md with explicit deny/permit branch and tenant/cell scope`.
5. Patch runtime config: `edit app/community/iac/kustomize/base/kustomization.yaml if deploy/config drift caused the incident`.
6. Add regression test: `cargo test -p community-moderation-queue-domain kb_attachment_restore_incident_regression -- --nocapture`.
7. Add gate evidence: `cargo run -p dev-cli -- gate validate community-kb-attachment-restore --fixture incident-kb-attachment-restore.json`.
8. Add SLO assertion: `update app/community/slos/audit-chain-seal-latency.openslo.yaml with alert KbAttachmentRestoreCritical when this was a missing alert`.
9. Add dashboard panel: `update app/community/dashboards/moderation-queue-depth.json with community_kb_attachment_restore_error_ratio, community_kb_attachment_restore_lag_seconds, and community_kb_attachment_restore_total`.
10. Rebuild affected crate: `cargo check -p community-moderation-queue-domain --all-targets`.
11. Run targeted tests: `cargo test -p community-moderation-queue-domain --all-features`.
12. Run policy validation: `cargo run -p dev-cli -- gate validate community-policy --microservice community`.
13. Deploy canary: `oya deploy canary --microservice community --component community-kb-attachment-restore-worker --cell $CELL --weight 1`.
14. Watch burn rate: `oya ops watch --metric community_kb_attachment_restore_error_ratio --threshold 0.005 --window 30m --cell $CELL`.
15. Close circuit breaker: `oya ops breaker close community-kb-attachment-restore-circuit-breaker --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
16. Unfreeze automation: `oya flags set oya.community.kb_attachment_restore.incident_hold=false --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
17. Resume promotion: recovery PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
18. Seal resolution audit: `oya audit-chain emit --event-class EVT_COMMUNITY_KB_ATTACHMENT_RESTORE_INCIDENT --incident $INCIDENT_ID --field resolution=complete --field runbook=kb-attachment-restore`.
19. Verify seal: `oya audit-chain verify --event-class EVT_COMMUNITY_KB_ATTACHMENT_RESTORE_INCIDENT --incident $INCIDENT_ID`.
20. Attach final evidence: `oya evidence attach --incident $INCIDENT_ID --file evidence/incidents/$INCIDENT_ID.json --kind final-resolution`.

### Code Paths To Inspect First
- `community-moderation-queue-domain`: inspect for `kb_attachment_restore` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `community-post-store-worker`: inspect for `kb_attachment_restore` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `community-voting-engine-domain`: inspect for `kb_attachment_restore` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `community-search-index-worker`: inspect for `kb_attachment_restore` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `app/community/contracts/openapi/community.yaml`: verify request/response or event contract only when incident evidence points there.
- `app/community/contracts/asyncapi/community-events.yaml`: verify request/response or event contract only when incident evidence points there.
- `app/community/contracts/proto/community.proto`: verify request/response or event contract only when incident evidence points there.
- `app/community/dashboards/moderation-queue-depth.json`: verify panel coverage for `community_kb_attachment_restore_error_ratio`, `community_kb_attachment_restore_lag_seconds`, and `community_kb_attachment_restore_total`.
- `app/community/slos/`: verify alert vocabulary and threshold alignment before changing runtime thresholds.
- `app/community/policy/`: verify policy branch ownership before relaxing deny rules or emergency bypasses.

## Verification Checklist
- `KbAttachmentRestoreCritical` and `KbAttachmentRestoreSloBurn` are both resolved in Alertmanager for 30 minutes.
- `community_kb_attachment_restore_error_ratio < 0.005` for 3 consecutive 10 minute windows.
- `community_kb_attachment_restore_lag_seconds < 120` for all production cells.
- `community_kb_attachment_restore_queue_depth` is draining and not growing for the affected tenant.
- Service-specific signal `community_kb_attachment_restore_total` is below the threshold documented in `app/community/slos/audit-chain-seal-latency.openslo.yaml`.
- Dashboard `https://grafana.dev.oyatie.internal/d/community-ops/kb-attachment-restore?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=101` shows green panels for the affected cell.
- Audit-chain query for `EVT_COMMUNITY_KB_ATTACHMENT_RESTORE_INCIDENT` returns mitigation and resolution events.
- Circuit breaker `community-kb-attachment-restore-circuit-breaker` is closed after rollback window.
- Feature flag `oya.community.kb_attachment_restore.incident_hold` is false for the affected tenant unless long-term hold is approved.
- Runbook invocation evidence is attached to `evidence/incidents/$INCIDENT_ID.json`.
- Service owner acknowledged final handoff in `#inc-community`.

## Capacity and Rollback Guardrails
- Capacity math: if `community_kb_attachment_restore_queue_depth` is 5000 and the worker drains 25 items/second, the best-case drain is 200 seconds before retries; page earlier when drain time exceeds 300 seconds.
- Capacity math: with 12 replicas at 25 items/second each, the hard ceiling is 300 items/second; keep tenant throttle below 25 RPS until error ratio stays below 0.005.
- Rollback checkpoint 1: before changing `oya.community.kb_attachment_restore.incident_hold`, snapshot current value with `oya flags get oya.community.kb_attachment_restore.incident_hold --output json`.
- Rollback checkpoint 2: before opening `community-kb-attachment-restore-circuit-breaker`, capture `community_kb_attachment_restore_request_rate` and `community_kb_attachment_restore_success_ratio` from Mimir.
- Rollback checkpoint 3: before scaling deployments, capture `kubectl -n community get deploy community-kb-attachment-restore-worker -o yaml`.
- Rollback command for flag: `oya flags set oya.community.kb_attachment_restore.incident_hold=false --cell $CELL --tenant $TENANT --reason rollback-$INCIDENT_ID`.
- Rollback command for breaker: `oya ops breaker close community-kb-attachment-restore-circuit-breaker --cell $CELL --tenant $TENANT --reason rollback-$INCIDENT_ID`.
- Rollback command for deployment: `kubectl -n community rollout undo deploy/community-kb-attachment-restore-worker`.
- Rollback command for tenant throttle: `oya ops rate-limit clear --tenant $TENANT --surface community.kb-attachment-restore --reason rollback-$INCIDENT_ID`.
- Stop rollback if `EVT_COMMUNITY_KB_ATTACHMENT_RESTORE_INCIDENT` cannot be emitted; preserve the current state and escalate to audit-chain before additional mutation.

## Postmortem Template
Use this exact skeleton for the incident document. The field names are intentionally stable for ADR-0263 audit emission extraction.
```markdown
---
doc_class: IncidentPostmortem
runbook_id: community-kb-attachment-restore
microservice: community
event_class: EVT_COMMUNITY_KB_ATTACHMENT_RESTORE_INCIDENT
incident_id: <INC-...>
severity: sev2
status: draft
detected_at: <UTC>
mitigated_at: <UTC>
resolved_at: <UTC>
commander: <handle>
evidence_hash: <sha256>
---

# Kb Attachment Restore postmortem

## Summary
- What happened in community/kb-attachment-restore.
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
- Emit EVT_COMMUNITY_KB_ATTACHMENT_RESTORE_INCIDENT with incident_id, tenant_id, cell_id, principal_id, decision_id, evidence_hash, operator_id, runbook_id.
- Attach dashboard snapshot URLs and command transcripts.
- Seal mitigation and resolution events before closure.

## Corrective Actions
- Action, owner, due date, validation command, linked issue.
```

## Escalation Path
- Primary on-call: PagerDuty community-primary; trust-safety-secondary.
- Incident SLA: ack 3m for sev0/sev1, 10m for sev2, 30m for sev3; status update every 10m until the critical alert clears.
- Incident commander: first responder from axis-community + ops-sre-reliability; transfer only by explicit message in `#inc-community`.
- Security escalation: page `ops-security-primary` immediately for sev0, credential, cross-tenant, fraud, or audit-seal symptoms.
- Compliance escalation: page `dpo-office-duty` when tenant data, regulator evidence, money movement, or breach-clock symptoms are present.
- Architecture escalation: page `council-architecture-reviewer` before manual bypass, policy rollback, or invariant relaxation.
- External vendors: Cloudflare Trust and Safety; OpenSearch support; Zendesk Trust Center support. Open a ticket once local dependency health is proven and vendor dependency remains suspect.
- Customer communications: use status page component `oyatie-community-kb-attachment-restore` and keep private details in the incident channel.
- Regulatory clock: if tenant data, financial correctness, or evidence integrity is possibly affected, start the compliance 72h assessment timer even if exposure is unconfirmed.
- Executive notice: sev0 or fleet-wide sev1 goes to `#exec-incident-readout` within 30 minutes.

## Cross-µservice Coordination
- Notify `identity`: `oya incident handoff --target identity --source community --runbook kb-attachment-restore --incident $INCIDENT_ID --severity sev2 --branch A`; expect `202 accepted`.
- Require `identity` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `tenancy`: `oya incident handoff --target tenancy --source community --runbook kb-attachment-restore --incident $INCIDENT_ID --severity sev2 --branch B`; expect `202 accepted`.
- Require `tenancy` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `compliance`: `oya incident handoff --target compliance --source community --runbook kb-attachment-restore --incident $INCIDENT_ID --severity sev2 --branch C`; expect `202 accepted`.
- Require `compliance` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `audit-chain`: `oya incident handoff --target audit-chain --source community --runbook kb-attachment-restore --incident $INCIDENT_ID --severity sev2 --branch D`; expect `202 accepted`.
- Require `audit-chain` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `observability`: `oya incident handoff --target observability --source community --runbook kb-attachment-restore --incident $INCIDENT_ID --severity sev2 --branch A`; expect `202 accepted`.
- Require `observability` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Observability handoff API: `oya incident handoff --target observability --source community --runbook kb-attachment-restore --incident $INCIDENT_ID`.
- Governance handoff API: `oya incident handoff --target governance --source community --runbook kb-attachment-restore --incident $INCIDENT_ID`.
- Compliance handoff API: `oya incident handoff --target compliance --source community --runbook kb-attachment-restore --incident $INCIDENT_ID`.
- Audit-chain handoff API: `oya incident handoff --target audit-chain --source community --runbook kb-attachment-restore --incident $INCIDENT_ID`.
- Tenancy handoff API: `oya incident handoff --target tenancy --source community --runbook kb-attachment-restore --incident $INCIDENT_ID`.

## Handoff Notes
- Do not hand off with only the alert name; include `community_kb_attachment_restore_error_ratio`, `community_kb_attachment_restore_lag_seconds`, `community_kb_attachment_restore_queue_depth`, `community_kb_attachment_restore_total`, current breaker state, and audit seal status.
- Keep `community-kb-attachment-restore-circuit-breaker` owner as axis-community + ops-sre-reliability until the receiving service explicitly accepts.
- If another runbook owns the downstream fix, link this incident as upstream and keep this runbook open until downstream verification returns green.
- Close only after `EVT_COMMUNITY_KB_ATTACHMENT_RESTORE_INCIDENT` has a sealed resolution row and every coordination endpoint above has either accepted or explicitly declined scope.

## Sources Checked During This Substance Pass
- `app/community/dashboards/` for dashboard names and operational panels: moderation-queue-depth.json, post-throughput.json, vote-rate.json.
- `app/community/slos/` for OpenSLO alert vocabulary and threshold alignment: audit-chain-seal-latency.openslo.yaml, feed-render-latency.openslo.yaml, kb-article-publish-latency.openslo.yaml, moderation-action-latency.openslo.yaml, post-create-latency.openslo.yaml, search-query-latency.openslo.yaml, vote-cast-latency.openslo.yaml.
- `app/community/policy/` for named policy and authorization surfaces: community-isolation.md, anonymity-mode-identity-anchored.cedar, anonymity-mode-fully-anonymous.cedar, tenant-scope.cedar.
- `app/community/contracts/` for API, AsyncAPI, proto, and adapter surfaces: contracts/openapi/community.yaml, contracts/asyncapi/community-events.yaml, contracts/proto/community.proto.
- `app/community/manifest.json` for owner, dependency, capability, and bounded-context vocabulary; topic `kb-attachment-restore` is the scenario anchor.

## Checkpoint Closure Criteria
- The runbook remains current when `KbAttachmentRestoreCritical`, `KbAttachmentRestoreSloBurn`, `community_kb_attachment_restore_total`, `oya.community.kb_attachment_restore.incident_hold`, and `community-kb-attachment-restore-circuit-breaker` all resolve to live telemetry, flag, or breaker records.
- The incident is cleanly halted if required authority is missing for tenant quarantine, policy rollback, or vendor escalation; do not improvise outside the named commands.
- The checkpoint is complete when `./bin/oya vcs verify --agent codex-runbooks-substrate-w3 --evidence 'runbooks_substance:X new_runbooks:Y' ...` accepts the five target scopes.
