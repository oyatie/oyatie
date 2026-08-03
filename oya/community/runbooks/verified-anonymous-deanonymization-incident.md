---
doc_class: Runbook
title: Verified Anonymous Deanonymization Incident
status: Accepted
date: 2026-05-20
microservice: community
severity: sev2
audience: community-trust-and-safety-on-call
owner_team: axis-community + ops-sre-reliability
source_wave: codex-runbooks-substrate-w3
change_scope: net-new runbook
doc_status: published
---

# Runbook: Verified Anonymous Deanonymization Incident

## Operator Contract
- Runbook id: community-verified-anonymous-deanonymization-incident.
- Primary service namespace: `community`.
- Owning rotation: PagerDuty oya-community-primary; trust-safety-secondary.
- Incident channel: `#inc-community`.
- Operational focus: verified anonymous member identity may have been exposed or correlated.
- Named precedent: this follows the Reddit moderation queue plus Stack Overflow vote-integrity and Cloudflare abuse-response pattern.
- External dependencies: Cloudflare Trust and Safety; OpenSearch support; Zendesk Trust Center support.
- API authority: `https://community.internal.oyatie.dev/v1/community/verified-anonymous-deanonymization-incident/incident-handoff`.
- Audit event class: `EVT_COMMUNITY_VERIFIED_ANONYMOUS_DEANONYMIZATION_INCIDENT` with ADR-0263 fields `incident_id`, `tenant_id`, `cell_id`, `microservice`, `runbook_id`, `decision_id`, `evidence_hash`, `operator_id`.
- Stop condition: mitigation has held for 30 minutes, `VerifiedAnonymousDeanonymizationIncidentCritical` is green, and every Cross-microservice handoff API returns `202 accepted`.
- Safety invariant: never clear the incident until `EVT_COMMUNITY_VERIFIED_ANONYMOUS_DEANONYMIZATION_INCIDENT` is sealed and the postmortem skeleton exists under `evidence/postmortems/community-verified-anonymous-deanonymization-incident-<incident-id>.md`.

## Trigger Conditions
- Page on alert `VerifiedAnonymousDeanonymizationIncidentCritical` when `oya_community_verified_anonymous_deanonymization_incident_error_ratio > 0.02` for 10 minutes in any production cell.
- Page on alert `VerifiedAnonymousDeanonymizationIncidentSloBurn` when `oya_community_verified_anonymous_deanonymization_incident_lag_seconds > 300` for 2 consecutive evaluator windows.
- Open sev2 if `oya_community_verified_anonymous_deanonymization_total` exceeds the threshold documented in `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`.
- Open sev2 if `oya_community_verified_anonymous_deanonymization_incident_queue_depth > 5000` for 15 minutes or retry backlog grows by more than 20 percent in one 5 minute window.
- Trigger from customer report when Support tags the case `community.verified-anonymous-deanonymization-incident.customer_visible` in Zendesk.
- Trigger from CI when `cargo run -p oya-dev-cli -- gate validate community-verified-anonymous-deanonymization-incident --production-snapshot` exits non-zero against the latest production evidence bundle.
- Primary dashboard: `https://grafana.dev.oyatie.internal/d/community-ops/verified-anonymous-deanonymization-incident?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=101` backed by `microservices/community/dashboards/moderation-queue-depth.json`.
- Secondary dashboard: `https://grafana.dev.oyatie.internal/d/community-ops/verified-anonymous-deanonymization-incident?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=202` backed by `microservices/community/dashboards/post-throughput.json`.
- Loki explorer: `https://grafana.dev.oyatie.internal/explore?query={namespace="community",runbook="verified-anonymous-deanonymization-incident"}`.
- Alertmanager route: `oyatie-community-verified-anonymous-deanonymization-incident-critical`; silence only with incident commander approval and `EVT_COMMUNITY_VERIFIED_ANONYMOUS_DEANONYMIZATION_INCIDENT` evidence.
- Synthetic probe: `oya ops probe community verified-anonymous-deanonymization-incident --cell prod-us-east-1 --tenant synthetic-canary` returns `healthy=true`.
- Drift detector: `registry/community/verified-anonymous-deanonymization-incident/expected-state.json` hash differs from live `https://community.internal.oyatie.dev/v1/community/verified-anonymous-deanonymization-incident/admin/state-hash`.
- Service-specific metric `oya_community_verified_anonymous_deanonymization_total` is red while `oya_community_verified_anonymous_deanonymization_incident_audit_emit_total{status="sealed"}` is flat.

## Symptoms
- User-facing impact: members, moderators, or tenant admins may see moderation, anonymity, search, voting, or post integrity failures; scenario focus is verified anonymous member identity may have been exposed or correlated.
- Operators see Grafana panel `moderation-queue-depth.json / Verified Anonymous Deanonymization Incident burn rate` turn red before the primary alert resolves.
- Loki signature `community.verified_anonymous_deanonymization_incident.incident_state=failed` appears with fields `incident_id`, `tenant_id`, `cell_id`, `decision_id`, `evidence_hash`.
- Kubernetes events include `reason=VerifiedAnonymousDeanonymizationIncidentDegraded` on deployment `community-verified-anonymous-deanonymization-incident-worker` or `community-api`.
- Audit-chain shows missing or delayed `EVT_COMMUNITY_VERIFIED_ANONYMOUS_DEANONYMIZATION_INCIDENT` entries when queried with `oya audit-chain query --event-class EVT_COMMUNITY_VERIFIED_ANONYMOUS_DEANONYMIZATION_INCIDENT --since 30m`.
- Metric pattern: `oya_community_verified_anonymous_deanonymization_incident_error_ratio` rises before `oya_community_verified_anonymous_deanonymization_incident_lag_seconds`; if lag rises first, suspect dependency saturation rather than local regression.
- Metric pattern: `oya_community_verified_anonymous_deanonymization_incident_queue_depth` increases while pod CPU stays below 40 percent; suspect downstream refusal, replay backlog, or feature flag deadlock.
- Tenant-specific shape: one `tenant_id` dominates labels in `oya_community_verified_anonymous_deanonymization_incident_queue_depth`; isolate tenant before fleet mitigation.
- Fleet-wide shape: at least three cells report `VerifiedAnonymousDeanonymizationIncidentCritical` in one 15 minute window; switch to cross-cell bridge even if individual tenants are low-volume.
- Log signature `decision=deny reason=verified-anonymous-deanonymization-incident.policy_guard` means the guard is working; investigate caller inputs before rollback.
- Log signature `decision=permit reason=verified-anonymous-deanonymization-incident.break_glass` means manual intervention is active; confirm two-person authorization.
- Log signature `audit_emit_status=stalled event_class=EVT_COMMUNITY_VERIFIED_ANONYMOUS_DEANONYMIZATION_INCIDENT` means mitigation cannot be closed until replay succeeds.
- Service-specific pattern: `oya_community_verified_anonymous_deanonymization_total` rises while `oya_community_verified_anonymous_deanonymization_incident_dependency_error_ratio` is flat; inspect local state before escalating Cloudflare Trust and Safety.
- Service-specific pattern: `oya_community_verified_anonymous_deanonymization_incident_dependency_error_ratio` rises while `oya_community_verified_anonymous_deanonymization_total` is flat; inspect vendor or adjacent-service dependency health before local rollback.

## Failure Mode Tree
- Failure mode 1: single-tenant CommunityThread inconsistency; contain with tenant quarantine, preserve all `EVT_COMMUNITY_VERIFIED_ANONYMOUS_DEANONYMIZATION_INCIDENT` rows, and avoid fleet rollback.
- Failure mode 2: cross-cell ModerationDecision drift; freeze writes, compare state hash across cells, and use audit-chain replay before accepting new mutations.
- Failure mode 3: byzantine or abusive principal; suspend the principal through identity, keep tenant data scoped, and preserve Cedar explain output.
- Failure mode 4: external dependency outage at Cloudflare Trust and Safety; open vendor ticket only after local dashboards and handoff APIs prove the dependency is causal.
- Failure mode 5: operator mitigation made state worse; roll back feature flag `oya.community.verified_anonymous_deanonymization_incident.incident_hold`, close `community-verified-anonymous-deanonymization-incident-circuit-breaker`, and restore the previous deployment revision.
- Failure mode 6: audit emission is delayed; do not close even when customer symptoms improve because ADR-0263 evidence is incomplete.
- Failure mode 7: regional partition; keep prod-us-east-1 as evidence leader and reject cross-region mutation until `oya_community_verified_anonymous_deanonymization_incident_state_hash_match == 1`.
- Failure mode 8: compliance-pack mismatch; require compliance handoff when KR-CSAP, EU-sovereign, FedRAMP-High, IL5, or CN-PIPL labels are present.
- Failure mode 9: stale dashboard data; verify direct Mimir queries before making rollback decisions.
- Failure mode 10: runbook step ambiguity; halt the ambiguous branch, emit `EVT_COMMUNITY_VERIFIED_ANONYMOUS_DEANONYMIZATION_INCIDENT` with outcome `blocked`, and patch this runbook after recovery.

## Diagnostic Steps
1. Set incident variables: `export INCIDENT_ID=INC-community-verified-anonymous-deanonymization-incident-$(date -u +%Y%m%dT%H%M%SZ); export CELL=prod-us-east-1; export TENANT=synthetic-canary`.
2. Confirm active alerts: `curl -s https://community.internal.oyatie.dev/v1/community/alerts?runbook=verified-anonymous-deanonymization-incident | jq .alerts`.
3. Check Kubernetes rollout: `kubectl -n community rollout status deploy/community-verified-anonymous-deanonymization-incident-worker --timeout=60s`.
4. List unhealthy pods: `kubectl -n community get pods -l app=verified-anonymous-deanonymization-incident -o wide`.
5. Read structured logs: `kubectl -n community logs deploy/community-verified-anonymous-deanonymization-incident-worker --since=30m | rg "community.verified_anonymous_deanonymization_incident.incident_state|VerifiedAnonymousDeanonymizationIncidentCritical|EVT_COMMUNITY_VERIFIED_ANONYMOUS_DEANONYMIZATION_INCIDENT"`.
6. Query Loki directly: `logcli query '{namespace="community",runbook="verified-anonymous-deanonymization-incident"}' --since=30m --limit=200`.
7. Check Prometheus error ratio: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_community_verified_anonymous_deanonymization_incident_error_ratio{cell="prod-us-east-1"}'`.
8. Check lag: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_community_verified_anonymous_deanonymization_incident_lag_seconds{cell="prod-us-east-1"}'`.
9. Check queue: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_community_verified_anonymous_deanonymization_incident_queue_depth{cell="prod-us-east-1"}'`.
10. Check service-specific signal: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_community_verified_anonymous_deanonymization_total{cell="prod-us-east-1"}'`.
11. Open primary dashboard: `open "https://grafana.dev.oyatie.internal/d/community-ops/verified-anonymous-deanonymization-incident?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=101&var-incident=$INCIDENT_ID"`.
12. Open secondary dashboard: `open "https://grafana.dev.oyatie.internal/d/community-ops/verified-anonymous-deanonymization-incident?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=202&var-tenant=$TENANT"`.
13. Verify audit-chain emission: `oya audit-chain query --event-class EVT_COMMUNITY_VERIFIED_ANONYMOUS_DEANONYMIZATION_INCIDENT --since 30m --cell $CELL --tenant $TENANT`.
14. Verify service state: `oya ops community verified-anonymous-deanonymization-incident status --cell $CELL --tenant $TENANT --output json`.
15. Run production snapshot gate: `cargo run -p oya-dev-cli -- gate validate community-verified-anonymous-deanonymization-incident --production-snapshot --cell $CELL`.
16. Run crate smoke test: `cargo test -p oya-community-moderation-queue-domain verified_anonymous_deanonymization_incident -- --nocapture`.
17. Check API contract smoke: `curl -s https://community.internal.oyatie.dev/v1/community/verified-anonymous-deanonymization-incident/incident-handoff -H "x-oya-tenant: $TENANT"`.
18. Inspect config: `test -f microservices/community/iac/kustomize/base/kustomization.yaml && sed -n '1,180p' microservices/community/iac/kustomize/base/kustomization.yaml`.
19. Inspect feature flags: `oya flags get oya.community.verified_anonymous_deanonymization_incident.incident_hold --cell $CELL --tenant $TENANT --output yaml`.
20. Inspect circuit breaker: `oya ops breaker status community-verified-anonymous-deanonymization-incident-circuit-breaker --cell $CELL --tenant $TENANT`.
21. Check recent deploy: `kubectl -n community rollout history deploy/community-verified-anonymous-deanonymization-incident-worker | tail -20`.
22. Check policy file: `test -f microservices/community/policy/community-isolation.md || find microservices/community/policy -maxdepth 2 -type f | sort`.
23. Check SLO files: `ls microservices/community/slos/*.openslo.yaml | sort | rg "audit|feed"`.
24. Check contract binding: `test -f microservices/community/contracts/openapi/community.yaml && sed -n '1,120p' microservices/community/contracts/openapi/community.yaml`.
25. Run targeted SQL state query: `psql $OYA_PROD_DSN -c "select incident_id, tenant_id, cell_id, state, updated_at from community_verified_anonymous_deanonymization_incidents where updated_at > now() - interval '30 minutes' order by updated_at desc limit 20;"`.
26. Confirm no cross-cell spread: `oya ops cells query --metric oya_community_verified_anonymous_deanonymization_incident_error_ratio --window 30m --threshold 0.02`.
27. Snapshot evidence: `oya evidence snapshot --incident $INCIDENT_ID --microservice community --runbook verified-anonymous-deanonymization-incident --output evidence/incidents/$INCIDENT_ID.json`.

### Diagnostic Decision Tree
```text
Verified Anonymous Deanonymization Incident decision tree
1. Is VerifiedAnonymousDeanonymizationIncidentCritical firing in more than one cell?
   |-- yes: declare fleet incident, page PagerDuty oya-community-primary, and run cross-cell containment.
   |-- no: keep scope to the affected cell and continue tenant isolation checks.
2. Does oya_community_verified_anonymous_deanonymization_incident_queue_depth grow while oya_community_verified_anonymous_deanonymization_incident_error_ratio is flat?
   |-- yes: downstream dependency, replay backlog, or queue-drain issue; choose mitigation branch B.
   |-- no: local regression, bad input, or policy/config drift; continue branch selection.
3. Does audit-chain show EVT_COMMUNITY_VERIFIED_ANONYMOUS_DEANONYMIZATION_INCIDENT gaps?
   |-- yes: do not close; run evidence replay before resolution.
   |-- no: mitigation can proceed after state is green.
4. Is customer, finance, security, or regulator impact confirmed?
   |-- yes: promote severity, open #inc-community, and notify compliance or security handoff.
   |-- no: keep internal incident and collect evidence.
```
- Branch A (confirmed CommunityThread correctness risk): use the matching mitigation block below and record `decision_branch=A` in `EVT_COMMUNITY_VERIFIED_ANONYMOUS_DEANONYMIZATION_INCIDENT`.
- Branch B (dependency saturation or replay backlog): use the matching mitigation block below and record `decision_branch=B` in `EVT_COMMUNITY_VERIFIED_ANONYMOUS_DEANONYMIZATION_INCIDENT`.
- Branch C (policy, permit, or tenant-scope drift): use the matching mitigation block below and record `decision_branch=C` in `EVT_COMMUNITY_VERIFIED_ANONYMOUS_DEANONYMIZATION_INCIDENT`.
- Branch D (customer-visible or regulated evidence gap): use the matching mitigation block below and record `decision_branch=D` in `EVT_COMMUNITY_VERIFIED_ANONYMOUS_DEANONYMIZATION_INCIDENT`.

## Mitigation Steps
1. Acknowledge page: `pd incident ack --service community --incident $INCIDENT_ID`.
2. Create bridge: `oya incident bridge create --incident $INCIDENT_ID --channel #inc-community --severity sev2`.
3. Freeze risky automation: `oya flags set oya.community.verified_anonymous_deanonymization_incident.incident_hold=true --cell $CELL --tenant $TENANT --reason $INCIDENT_ID`.
4. Enable circuit breaker: `oya ops breaker open community-verified-anonymous-deanonymization-incident-circuit-breaker --cell $CELL --tenant $TENANT --ttl 30m --reason $INCIDENT_ID`.
5. Reduce blast radius: `kubectl -n community scale deploy/community-verified-anonymous-deanonymization-incident-worker --replicas=1`.
6. Protect tenant boundary: `oya tenancy quarantine --tenant $TENANT --reason community-verified-anonymous-deanonymization-incident --ttl 60m`.
7. Pause promotion: incident hold PR against `dev` (plain `git`; branch-protected `oya-ci-required` required; legacy `oya gate` output optional local/provenance only).
8. Drain queue safely: `oya ops community verified-anonymous-deanonymization-incident drain --cell $CELL --tenant $TENANT --max-items 500 --dry-run`.
9. Execute bounded drain: `oya ops community verified-anonymous-deanonymization-incident drain --cell $CELL --tenant $TENANT --max-items 500 --confirm $INCIDENT_ID`.
10. Replay missing audit events: `oya audit-chain replay --event-class EVT_COMMUNITY_VERIFIED_ANONYMOUS_DEANONYMIZATION_INCIDENT --incident $INCIDENT_ID --from evidence/incidents/$INCIDENT_ID.json`.
11. Rollback last deploy if causal: `kubectl -n community rollout undo deploy/community-verified-anonymous-deanonymization-incident-worker`.
12. Raise HPA cap if saturation is proven: `kubectl -n community patch hpa community-verified-anonymous-deanonymization-incident-worker --type merge -p '{"spec":{"maxReplicas":12}}'`.
13. Throttle hot tenant: `oya ops rate-limit set --tenant $TENANT --surface community.verified-anonymous-deanonymization-incident --rps 25 --ttl 30m`.
14. Block abusive principal when relevant: `oya identity principal suspend --principal suspected-abuse --tenant $TENANT --reason $INCIDENT_ID`.
15. Protect evidence: `oya evidence freeze --incident $INCIDENT_ID --paths microservices/community/runbooks/verified-anonymous-deanonymization-incident.md,evidence/incidents/$INCIDENT_ID.json`.
16. Notify service owners: `oya notify service-owner --microservice community --incident $INCIDENT_ID --channel #inc-community`.
17. Open external vendor ticket: `oya vendor ticket open --vendor "Cloudflare Trust and Safety" --incident $INCIDENT_ID --summary community-verified-anonymous-deanonymization-incident`.
18. Confirm breaker effect: `oya ops breaker status community-verified-anonymous-deanonymization-incident-circuit-breaker --cell $CELL --tenant $TENANT --expect open`.
19. Confirm user impact reduced: `curl -s https://community.internal.oyatie.dev/v1/community/verified-anonymous-deanonymization-incident/incident-handoff/health -H "x-oya-tenant: $TENANT"`.
20. Emit mitigation audit: `oya audit-chain emit --event-class EVT_COMMUNITY_VERIFIED_ANONYMOUS_DEANONYMIZATION_INCIDENT --incident $INCIDENT_ID --field mitigation=active --field runbook=verified-anonymous-deanonymization-incident`.

### Mitigation Branch Guidance
- Branch A: confirmed CommunityThread correctness risk.
  - Required action: keep `community-verified-anonymous-deanonymization-incident-circuit-breaker` open until `oya_community_verified_anonymous_deanonymization_incident_error_ratio` is below 0.005 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/community-ops/verified-anonymous-deanonymization-incident?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=110` to the incident.
  - Required audit: emit `EVT_COMMUNITY_VERIFIED_ANONYMOUS_DEANONYMIZATION_INCIDENT` with `branch=A`, `operator_id`, and `evidence_hash`.
- Branch B: dependency saturation or replay backlog.
  - Required action: keep `community-verified-anonymous-deanonymization-incident-circuit-breaker` open until `oya_community_verified_anonymous_deanonymization_incident_error_ratio` is below 0.01 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/community-ops/verified-anonymous-deanonymization-incident?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=111` to the incident.
  - Required audit: emit `EVT_COMMUNITY_VERIFIED_ANONYMOUS_DEANONYMIZATION_INCIDENT` with `branch=B`, `operator_id`, and `evidence_hash`.
- Branch C: policy, permit, or tenant-scope drift.
  - Required action: keep `community-verified-anonymous-deanonymization-incident-circuit-breaker` open until `oya_community_verified_anonymous_deanonymization_incident_error_ratio` is below 0.005 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/community-ops/verified-anonymous-deanonymization-incident?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=112` to the incident.
  - Required audit: emit `EVT_COMMUNITY_VERIFIED_ANONYMOUS_DEANONYMIZATION_INCIDENT` with `branch=C`, `operator_id`, and `evidence_hash`.
- Branch D: customer-visible or regulated evidence gap.
  - Required action: keep `community-verified-anonymous-deanonymization-incident-circuit-breaker` open until `oya_community_verified_anonymous_deanonymization_incident_error_ratio` is below 0.01 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/community-ops/verified-anonymous-deanonymization-incident?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=113` to the incident.
  - Required audit: emit `EVT_COMMUNITY_VERIFIED_ANONYMOUS_DEANONYMIZATION_INCIDENT` with `branch=D`, `operator_id`, and `evidence_hash`.

## Resolution Steps
1. Identify code owner path: `rg "verified_anonymous_deanonymization_incident|VerifiedAnonymousDeanonymizationIncidentCritical|community.verified_anonymous_deanonymization_incident.incident_state" crates microservices/community -g "!microservices/community/runbooks/**"`.
2. Patch domain invariant: `edit oya-community-moderation-queue-domain where verified_anonymous_deanonymization_incident state transition is validated`.
3. Patch API guard: `edit microservices/community/contracts/openapi/community.yaml if the failing path is north-south or async handoff`.
4. Patch policy: `edit microservices/community/policy/community-isolation.md with explicit deny/permit branch and tenant/cell scope`.
5. Patch runtime config: `edit microservices/community/iac/kustomize/base/kustomization.yaml if deploy/config drift caused the incident`.
6. Add regression test: `cargo test -p oya-community-moderation-queue-domain verified_anonymous_deanonymization_incident_incident_regression -- --nocapture`.
7. Add gate evidence: `cargo run -p oya-dev-cli -- gate validate community-verified-anonymous-deanonymization-incident --fixture incident-verified-anonymous-deanonymization-incident.json`.
8. Add SLO assertion: `update microservices/community/slos/audit-chain-seal-latency.openslo.yaml with alert VerifiedAnonymousDeanonymizationIncidentCritical when this was a missing alert`.
9. Add dashboard panel: `update microservices/community/dashboards/moderation-queue-depth.json with oya_community_verified_anonymous_deanonymization_incident_error_ratio, oya_community_verified_anonymous_deanonymization_incident_lag_seconds, and oya_community_verified_anonymous_deanonymization_total`.
10. Rebuild affected crate: `cargo check -p oya-community-moderation-queue-domain --all-targets`.
11. Run targeted tests: `cargo test -p oya-community-moderation-queue-domain --all-features`.
12. Run policy validation: `cargo run -p oya-dev-cli -- gate validate community-policy --microservice community`.
13. Deploy canary: `oya deploy canary --microservice community --component community-verified-anonymous-deanonymization-incident-worker --cell $CELL --weight 1`.
14. Watch burn rate: `oya ops watch --metric oya_community_verified_anonymous_deanonymization_incident_error_ratio --threshold 0.005 --window 30m --cell $CELL`.
15. Close circuit breaker: `oya ops breaker close community-verified-anonymous-deanonymization-incident-circuit-breaker --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
16. Unfreeze automation: `oya flags set oya.community.verified_anonymous_deanonymization_incident.incident_hold=false --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
17. Resume promotion: recovery PR against `dev` (plain `git`; branch-protected `oya-ci-required` required; legacy `oya gate` output optional local/provenance only).
18. Seal resolution audit: `oya audit-chain emit --event-class EVT_COMMUNITY_VERIFIED_ANONYMOUS_DEANONYMIZATION_INCIDENT --incident $INCIDENT_ID --field resolution=complete --field runbook=verified-anonymous-deanonymization-incident`.
19. Verify seal: `oya audit-chain verify --event-class EVT_COMMUNITY_VERIFIED_ANONYMOUS_DEANONYMIZATION_INCIDENT --incident $INCIDENT_ID`.
20. Attach final evidence: `oya evidence attach --incident $INCIDENT_ID --file evidence/incidents/$INCIDENT_ID.json --kind final-resolution`.

### Code Paths To Inspect First
- `oya-community-moderation-queue-domain`: inspect for `verified_anonymous_deanonymization_incident` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `oya-community-post-store-worker`: inspect for `verified_anonymous_deanonymization_incident` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `oya-community-voting-engine-domain`: inspect for `verified_anonymous_deanonymization_incident` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `oya-community-search-index-worker`: inspect for `verified_anonymous_deanonymization_incident` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `microservices/community/contracts/openapi/community.yaml`: verify request/response or event contract only when incident evidence points there.
- `microservices/community/contracts/asyncapi/community-events.yaml`: verify request/response or event contract only when incident evidence points there.
- `microservices/community/contracts/proto/community.proto`: verify request/response or event contract only when incident evidence points there.
- `microservices/community/dashboards/moderation-queue-depth.json`: verify panel coverage for `oya_community_verified_anonymous_deanonymization_incident_error_ratio`, `oya_community_verified_anonymous_deanonymization_incident_lag_seconds`, and `oya_community_verified_anonymous_deanonymization_total`.
- `microservices/community/slos/`: verify alert vocabulary and threshold alignment before changing runtime thresholds.
- `microservices/community/policy/`: verify policy branch ownership before relaxing deny rules or emergency bypasses.

## Verification Checklist
- `VerifiedAnonymousDeanonymizationIncidentCritical` and `VerifiedAnonymousDeanonymizationIncidentSloBurn` are both resolved in Alertmanager for 30 minutes.
- `oya_community_verified_anonymous_deanonymization_incident_error_ratio < 0.005` for 3 consecutive 10 minute windows.
- `oya_community_verified_anonymous_deanonymization_incident_lag_seconds < 120` for all production cells.
- `oya_community_verified_anonymous_deanonymization_incident_queue_depth` is draining and not growing for the affected tenant.
- Service-specific signal `oya_community_verified_anonymous_deanonymization_total` is below the threshold documented in `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`.
- Dashboard `https://grafana.dev.oyatie.internal/d/community-ops/verified-anonymous-deanonymization-incident?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=101` shows green panels for the affected cell.
- Audit-chain query for `EVT_COMMUNITY_VERIFIED_ANONYMOUS_DEANONYMIZATION_INCIDENT` returns mitigation and resolution events.
- Circuit breaker `community-verified-anonymous-deanonymization-incident-circuit-breaker` is closed after rollback window.
- Feature flag `oya.community.verified_anonymous_deanonymization_incident.incident_hold` is false for the affected tenant unless long-term hold is approved.
- Runbook invocation evidence is attached to `evidence/incidents/$INCIDENT_ID.json`.
- Service owner acknowledged final handoff in `#inc-community`.

## Capacity and Rollback Guardrails
- Capacity math: if `oya_community_verified_anonymous_deanonymization_incident_queue_depth` is 5000 and the worker drains 25 items/second, the best-case drain is 200 seconds before retries; page earlier when drain time exceeds 300 seconds.
- Capacity math: with 12 replicas at 25 items/second each, the hard ceiling is 300 items/second; keep tenant throttle below 25 RPS until error ratio stays below 0.005.
- Rollback checkpoint 1: before changing `oya.community.verified_anonymous_deanonymization_incident.incident_hold`, snapshot current value with `oya flags get oya.community.verified_anonymous_deanonymization_incident.incident_hold --output json`.
- Rollback checkpoint 2: before opening `community-verified-anonymous-deanonymization-incident-circuit-breaker`, capture `oya_community_verified_anonymous_deanonymization_incident_request_rate` and `oya_community_verified_anonymous_deanonymization_incident_success_ratio` from Mimir.
- Rollback checkpoint 3: before scaling deployments, capture `kubectl -n community get deploy community-verified-anonymous-deanonymization-incident-worker -o yaml`.
- Rollback command for flag: `oya flags set oya.community.verified_anonymous_deanonymization_incident.incident_hold=false --cell $CELL --tenant $TENANT --reason rollback-$INCIDENT_ID`.
- Rollback command for breaker: `oya ops breaker close community-verified-anonymous-deanonymization-incident-circuit-breaker --cell $CELL --tenant $TENANT --reason rollback-$INCIDENT_ID`.
- Rollback command for deployment: `kubectl -n community rollout undo deploy/community-verified-anonymous-deanonymization-incident-worker`.
- Rollback command for tenant throttle: `oya ops rate-limit clear --tenant $TENANT --surface community.verified-anonymous-deanonymization-incident --reason rollback-$INCIDENT_ID`.
- Stop rollback if `EVT_COMMUNITY_VERIFIED_ANONYMOUS_DEANONYMIZATION_INCIDENT` cannot be emitted; preserve the current state and escalate to audit-chain before additional mutation.

## Postmortem Template
Use this exact skeleton for the incident document. The field names are intentionally stable for ADR-0263 audit emission extraction.
```markdown
---
doc_class: IncidentPostmortem
runbook_id: community-verified-anonymous-deanonymization-incident
microservice: community
event_class: EVT_COMMUNITY_VERIFIED_ANONYMOUS_DEANONYMIZATION_INCIDENT
incident_id: <INC-...>
severity: sev2
status: draft
detected_at: <UTC>
mitigated_at: <UTC>
resolved_at: <UTC>
commander: <handle>
evidence_hash: <sha256>
---

# Verified Anonymous Deanonymization Incident postmortem

## Summary
- What happened in community/verified-anonymous-deanonymization-incident.
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
- Emit EVT_COMMUNITY_VERIFIED_ANONYMOUS_DEANONYMIZATION_INCIDENT with incident_id, tenant_id, cell_id, principal_id, decision_id, evidence_hash, operator_id, runbook_id.
- Attach dashboard snapshot URLs and command transcripts.
- Seal mitigation and resolution events before closure.

## Corrective Actions
- Action, owner, due date, validation command, linked issue.
```

## Escalation Path
- Primary on-call: PagerDuty oya-community-primary; trust-safety-secondary.
- Incident SLA: ack 3m for sev0/sev1, 10m for sev2, 30m for sev3; status update every 10m until the critical alert clears.
- Incident commander: first responder from axis-community + ops-sre-reliability; transfer only by explicit message in `#inc-community`.
- Security escalation: page `ops-security-primary` immediately for sev0, credential, cross-tenant, fraud, or audit-seal symptoms.
- Compliance escalation: page `dpo-office-duty` when tenant data, regulator evidence, money movement, or breach-clock symptoms are present.
- Architecture escalation: page `council-architecture-reviewer` before manual bypass, policy rollback, or invariant relaxation.
- External vendors: Cloudflare Trust and Safety; OpenSearch support; Zendesk Trust Center support. Open a ticket once local dependency health is proven and vendor dependency remains suspect.
- Customer communications: use status page component `oyatie-community-verified-anonymous-deanonymization-incident` and keep private details in the incident channel.
- Regulatory clock: if tenant data, financial correctness, or evidence integrity is possibly affected, start the compliance 72h assessment timer even if exposure is unconfirmed.
- Executive notice: sev0 or fleet-wide sev1 goes to `#exec-incident-readout` within 30 minutes.

## Cross-µservice Coordination
- Notify `identity`: `oya incident handoff --target identity --source community --runbook verified-anonymous-deanonymization-incident --incident $INCIDENT_ID --severity sev2 --branch A`; expect `202 accepted`.
- Require `identity` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `tenancy`: `oya incident handoff --target tenancy --source community --runbook verified-anonymous-deanonymization-incident --incident $INCIDENT_ID --severity sev2 --branch B`; expect `202 accepted`.
- Require `tenancy` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `compliance`: `oya incident handoff --target compliance --source community --runbook verified-anonymous-deanonymization-incident --incident $INCIDENT_ID --severity sev2 --branch C`; expect `202 accepted`.
- Require `compliance` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `audit-chain`: `oya incident handoff --target audit-chain --source community --runbook verified-anonymous-deanonymization-incident --incident $INCIDENT_ID --severity sev2 --branch D`; expect `202 accepted`.
- Require `audit-chain` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `observability`: `oya incident handoff --target observability --source community --runbook verified-anonymous-deanonymization-incident --incident $INCIDENT_ID --severity sev2 --branch A`; expect `202 accepted`.
- Require `observability` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Observability handoff API: `oya incident handoff --target observability --source community --runbook verified-anonymous-deanonymization-incident --incident $INCIDENT_ID`.
- Governance handoff API: `oya incident handoff --target governance --source community --runbook verified-anonymous-deanonymization-incident --incident $INCIDENT_ID`.
- Compliance handoff API: `oya incident handoff --target compliance --source community --runbook verified-anonymous-deanonymization-incident --incident $INCIDENT_ID`.
- Audit-chain handoff API: `oya incident handoff --target audit-chain --source community --runbook verified-anonymous-deanonymization-incident --incident $INCIDENT_ID`.
- Tenancy handoff API: `oya incident handoff --target tenancy --source community --runbook verified-anonymous-deanonymization-incident --incident $INCIDENT_ID`.

## Handoff Notes
- Do not hand off with only the alert name; include `oya_community_verified_anonymous_deanonymization_incident_error_ratio`, `oya_community_verified_anonymous_deanonymization_incident_lag_seconds`, `oya_community_verified_anonymous_deanonymization_incident_queue_depth`, `oya_community_verified_anonymous_deanonymization_total`, current breaker state, and audit seal status.
- Keep `community-verified-anonymous-deanonymization-incident-circuit-breaker` owner as axis-community + ops-sre-reliability until the receiving service explicitly accepts.
- If another runbook owns the downstream fix, link this incident as upstream and keep this runbook open until downstream verification returns green.
- Close only after `EVT_COMMUNITY_VERIFIED_ANONYMOUS_DEANONYMIZATION_INCIDENT` has a sealed resolution row and every coordination endpoint above has either accepted or explicitly declined scope.

## Sources Checked During This Substance Pass
- `microservices/community/dashboards/` for dashboard names and operational panels: moderation-queue-depth.json, post-throughput.json, vote-rate.json.
- `microservices/community/slos/` for OpenSLO alert vocabulary and threshold alignment: audit-chain-seal-latency.openslo.yaml, feed-render-latency.openslo.yaml, kb-article-publish-latency.openslo.yaml, moderation-action-latency.openslo.yaml, post-create-latency.openslo.yaml, search-query-latency.openslo.yaml, vote-cast-latency.openslo.yaml.
- `microservices/community/policy/` for named policy and authorization surfaces: community-isolation.md, anonymity-mode-identity-anchored.cedar, anonymity-mode-fully-anonymous.cedar, tenant-scope.cedar.
- `microservices/community/contracts/` for API, AsyncAPI, proto, and adapter surfaces: contracts/openapi/community.yaml, contracts/asyncapi/community-events.yaml, contracts/proto/community.proto.
- `microservices/community/manifest.json` for owner, dependency, capability, and bounded-context vocabulary; topic `verified-anonymous-deanonymization-incident` is the scenario anchor.

## Checkpoint Closure Criteria
- The runbook remains current when `VerifiedAnonymousDeanonymizationIncidentCritical`, `VerifiedAnonymousDeanonymizationIncidentSloBurn`, `oya_community_verified_anonymous_deanonymization_total`, `oya.community.verified_anonymous_deanonymization_incident.incident_hold`, and `community-verified-anonymous-deanonymization-incident-circuit-breaker` all resolve to live telemetry, flag, or breaker records.
- The incident is cleanly halted if required authority is missing for tenant quarantine, policy rollback, or vendor escalation; do not improvise outside the named commands.
- The checkpoint is complete when `./bin/oya vcs verify --agent codex-runbooks-substrate-w3 --evidence 'runbooks_substance:X new_runbooks:Y' ...` accepts the five target scopes.
