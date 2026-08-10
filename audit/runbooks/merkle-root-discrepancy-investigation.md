---
doc_class: Runbook
title: Merkle Root Discrepancy Investigation
status: Accepted
date: 2026-05-20
microservice: audit-chain
severity: sev1
audience: compliance-officer
owner_team: axis-audit-chain + ops-sre-reliability + ops-security
source_wave: codex-runbooks-substrate-w2
change_scope: net-new operational scenario
doc_status: published
---

# Runbook: Merkle Root Discrepancy Investigation

## Operator Contract
- Runbook id: audit-chain-merkle-root-discrepancy-investigation.
- Primary service namespace: `audit-chain`.
- Owning rotation: PagerDuty oya-audit-chain-primary; compliance-evidence secondary.
- Incident channel: `#inc-audit-chain`.
- Operational focus: protecting the append-only audit chain while resolving merkle root discrepancy investigation without weakening Merkle, signature, HSM, or retention invariants.
- External dependencies: Thales Luna HSM support; DigiCert timestamp authority support; Oracle PostgreSQL support.
- API authority: `https://audit-chain.internal.oyatie.dev/v1/audit-chain/merkle-root-discrepancy-investigation/incident-handoff`.
- Audit event class: `EVT_AUDIT_CHAIN_MERKLE_ROOT_DISCREPANCY_INVESTIGATION_INCIDENT` with ADR-0263 fields `incident_id`, `tenant_id`, `cell_id`, `microservice`, `runbook_id`, `decision_id`, `evidence_hash`, `operator_id`.
- Stop condition: mitigation has held for 30 minutes, `AuditChainMerkleRootDiscrepancyInvestigationCritical` is green, and every handoff API in Cross-microservice Coordination returns `202 accepted`.
- Safety invariant: never clear the incident until `EVT_AUDIT_CHAIN_MERKLE_ROOT_DISCREPANCY_INVESTIGATION_INCIDENT` is sealed and the postmortem skeleton exists under `evidence/postmortems/audit-chain-merkle-root-discrepancy-investigation-<incident-id>.md`.

## Trigger Conditions
- Page on alert `AuditChainMerkleRootDiscrepancyInvestigationCritical` when `oya_audit_chain_merkle_root_discrepancy_investigation_error_ratio > 0.02` for 10 minutes in any production cell.
- Page on alert `AuditChainMerkleRootDiscrepancyInvestigationSloBurn` when `oya_audit_chain_merkle_root_discrepancy_investigation_lag_seconds > 300` for 2 consecutive evaluator windows.
- Open a sev1 if `oya_audit_chain_merkle_root_discrepancy_investigation_correctness_ratio < 0.9999` and the affected label set includes `tenant_id`, `cell_id`, or `principal_id`.
- Open a sev1 if `oya_audit_chain_merkle_root_discrepancy_investigation_queue_depth > 5000` for 15 minutes or retry backlog grows by more than 20 percent in one 5 minute window.
- Trigger from customer report when Support tags the case `audit-chain.merkle-root-discrepancy-investigation.customer_visible` in Zendesk.
- Trigger from CI when `cargo run -p oya-dev-cli -- gate validate audit-chain-merkle-root-discrepancy-investigation --production-snapshot` exits non-zero against the latest production evidence bundle.
- Primary dashboard: `https://grafana.dev.oyatie.internal/d/audit-chain-substrate/merkle-root-discrepancy-investigation?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=116`.
- Secondary dashboard: `https://grafana.dev.oyatie.internal/d/audit-chain-substrate/merkle-root-discrepancy-investigation?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=213`.
- Loki explorer: `https://grafana.dev.oyatie.internal/explore?query={namespace="audit-chain",runbook="merkle-root-discrepancy-investigation"}`.
- Alertmanager route: `oyatie-audit-chain-merkle-root-discrepancy-investigation-critical`; silence only with incident commander approval and `EVT_AUDIT_CHAIN_MERKLE_ROOT_DISCREPANCY_INVESTIGATION_INCIDENT` evidence.
- Synthetic probe: `oya ops probe audit-chain merkle-root-discrepancy-investigation --cell prod-us-east-1 --tenant synthetic-canary` returns `healthy=true`.
- Drift detector: `registry/audit-chain/merkle-root-discrepancy-investigation/expected-state.json` hash differs from live `https://audit-chain.internal.oyatie.dev/v1/audit-chain/admin/state-hash`.
- Service-specific metric `oya_audit_chain_merkle_root_discrepancy_investigation_merkle_gap_total` exceeds the threshold documented in `audit/observability/slos/chain-of-custody-integrity-correctness.openslo.yaml`.

## Symptoms
- User-facing impact: regulator evidence, customer audit exports, and internal chain-of-custody proofs may be delayed or unverifiable.
- Operators see Grafana panel `emission-rate.json / Merkle Root Discrepancy Investigation burn rate` turn red before the primary alert resolves.
- Loki signature `audit_chain.merkle_root_discrepancy_investigation.incident_state=failed` appears with fields `incident_id`, `tenant_id`, `cell_id`, `decision_id`, `evidence_hash`.
- Kubernetes events include `reason=AuditChainMerkleRootDiscrepancyInvestigationDegraded` on deployment `audit-chain-merkle-root-discrepancy-investigation-worker` or `audit-chain-sealing-worker`.
- Audit-chain shows missing or delayed `EVT_AUDIT_CHAIN_MERKLE_ROOT_DISCREPANCY_INVESTIGATION_INCIDENT` entries when queried with `oya audit-chain query --event-class EVT_AUDIT_CHAIN_MERKLE_ROOT_DISCREPANCY_INVESTIGATION_INCIDENT --since 30m`.
- Metric pattern: `oya_audit_chain_merkle_root_discrepancy_investigation_error_ratio` rises before `oya_audit_chain_merkle_root_discrepancy_investigation_lag_seconds`; if lag rises first, suspect dependency saturation rather than local regression.
- Metric pattern: `oya_audit_chain_merkle_root_discrepancy_investigation_queue_depth` increases while pod CPU stays below 40 percent; suspect downstream refusal, replay backlog, or feature flag deadlock.
- Tenant-specific shape: one `tenant_id` dominates labels in `oya_audit_chain_merkle_root_discrepancy_investigation_queue_depth`; isolate tenant before fleet mitigation.
- Fleet-wide shape: at least three cells report `AuditChainMerkleRootDiscrepancyInvestigationCritical` in one 15 minute window; switch to cross-cell bridge even if individual tenants are low-volume.
- Log signature `decision=deny reason=merkle-root-discrepancy-investigation.policy_guard` means the guard is working; investigate caller inputs before rollback.
- Log signature `decision=permit reason=merkle-root-discrepancy-investigation.break_glass` means manual intervention is active; confirm two-person authorization.
- Log signature `audit_emit_status=stalled event_class=EVT_AUDIT_CHAIN_MERKLE_ROOT_DISCREPANCY_INVESTIGATION_INCIDENT` means mitigation cannot be closed until replay succeeds.
- Service-specific metric pattern: `oya_audit_chain_merkle_root_discrepancy_investigation_seal_latency_seconds` rises while `oya_audit_chain_merkle_root_discrepancy_investigation_evidence_export_age_seconds` is flat; inspect local worker health before escalating vendors.
- Service-specific metric pattern: `oya_audit_chain_merkle_root_discrepancy_investigation_evidence_export_age_seconds` rises while `oya_audit_chain_merkle_root_discrepancy_investigation_error_ratio` is flat; suspect stale export, stale recommendation, stale projection, or vendor dependency lag.

## Diagnostic Steps
1. Set incident variables: `export INCIDENT_ID=INC-audit-chain-merkle-root-discrepancy-investigation-$(date -u +%Y%m%dT%H%M%SZ); export CELL=prod-us-east-1; export TENANT=synthetic-canary`.
2. Confirm active alerts: `curl -s https://audit-chain.internal.oyatie.dev/v1/audit-chain/alerts?runbook=merkle-root-discrepancy-investigation | jq .alerts`.
3. Check Kubernetes rollout: `kubectl -n audit-chain rollout status deploy/audit-chain-merkle-root-discrepancy-investigation-worker --timeout=60s`.
4. List unhealthy pods: `kubectl -n audit-chain get pods -l app=merkle-root-discrepancy-investigation -o wide`.
5. Read structured logs: `kubectl -n audit-chain logs deploy/audit-chain-merkle-root-discrepancy-investigation-worker --since=30m | rg "audit_chain.merkle_root_discrepancy_investigation.incident_state|AuditChainMerkleRootDiscrepancyInvestigationCritical|EVT_AUDIT_CHAIN_MERKLE_ROOT_DISCREPANCY_INVESTIGATION_INCIDENT"`.
6. Query Loki directly: `logcli query '{namespace="audit-chain",runbook="merkle-root-discrepancy-investigation"}' --since=30m --limit=200`.
7. Check Prometheus error ratio: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_audit_chain_merkle_root_discrepancy_investigation_error_ratio{cell="prod-us-east-1"}'`.
8. Check lag: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_audit_chain_merkle_root_discrepancy_investigation_lag_seconds{cell="prod-us-east-1"}'`.
9. Check queue: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_audit_chain_merkle_root_discrepancy_investigation_queue_depth{cell="prod-us-east-1"}'`.
10. Check service-specific signal: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_audit_chain_merkle_root_discrepancy_investigation_merkle_gap_total{cell="prod-us-east-1"}'`.
11. Open primary dashboard: `open "https://grafana.dev.oyatie.internal/d/audit-chain-substrate/merkle-root-discrepancy-investigation?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=116&var-incident=$INCIDENT_ID"`.
12. Open secondary dashboard: `open "https://grafana.dev.oyatie.internal/d/audit-chain-substrate/merkle-root-discrepancy-investigation?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=213&var-tenant=$TENANT"`.
13. Verify audit-chain emission: `oya audit-chain query --event-class EVT_AUDIT_CHAIN_MERKLE_ROOT_DISCREPANCY_INVESTIGATION_INCIDENT --since 30m --cell $CELL --tenant $TENANT`.
14. Verify service state: `oya ops audit-chain merkle-root-discrepancy-investigation status --cell $CELL --tenant $TENANT --output json`.
15. Run production snapshot gate: `cargo run -p oya-dev-cli -- gate validate audit-chain-merkle-root-discrepancy-investigation --production-snapshot --cell $CELL`.
16. Run crate smoke test: `cargo test -p oya-audit-chain-domain merkle_root_discrepancy_investigation -- --nocapture`.
17. Check API contract smoke: `curl -s https://audit-chain.internal.oyatie.dev/v1/audit-chain/merkle-root-discrepancy-investigation/incident-handoff -H "x-oya-tenant: $TENANT"`.
18. Inspect config: `test -f audit/iac/kustomize/base/kustomization.yaml && sed -n '1,180p' audit/iac/kustomize/base/kustomization.yaml`.
19. Inspect feature flags: `oya flags get oya.audit-chain.merkle_root_discrepancy_investigation.incident_hold --cell $CELL --tenant $TENANT --output yaml`.
20. Inspect circuit breaker: `oya ops breaker status audit-chain-merkle-root-discrepancy-investigation-circuit-breaker --cell $CELL --tenant $TENANT`.
21. Check recent deploy: `kubectl -n audit-chain rollout history deploy/audit-chain-merkle-root-discrepancy-investigation-worker | tail -20`.
22. Check policy file: `test -f microservices/audit-chain/policy/seal-integrity.md || find audit/policy -maxdepth 2 -type f | sort`.
23. Check SLO files: `ls audit/observability/slos/*.openslo.yaml | sort | rg "chain|merkle"`.
24. Check catalog components: `find audit/catalog -maxdepth 1 -type f | sort | rg "emission|sealing|verification|query|retention"`.
25. Run targeted SQL state query: `psql $OYA_PROD_DSN -c "select incident_id, tenant_id, cell_id, state, updated_at from audit_chain_merkle_root_discrepancy_investigation_incidents where updated_at > now() - interval '30 minutes' order by updated_at desc limit 20;"`.
26. Confirm no cross-cell spread: `oya ops cells query --metric oya_audit_chain_merkle_root_discrepancy_investigation_error_ratio --window 30m --threshold 0.02`.
27. Snapshot evidence: `oya evidence snapshot --incident $INCIDENT_ID --microservice audit-chain --runbook merkle-root-discrepancy-investigation --output evidence/incidents/$INCIDENT_ID.json`.

### Diagnostic Decision Tree
```text
Merkle Root Discrepancy Investigation incident decision tree
1. Is AuditChainMerkleRootDiscrepancyInvestigationCritical firing in more than one cell?
   |-- yes: declare fleet incident, page PagerDuty oya-audit-chain-primary; compliance-evidence secondary, and run cross-cell containment.
   |-- no: keep scope to the affected cell and continue tenant isolation checks.
2. Does oya_audit_chain_merkle_root_discrepancy_investigation_queue_depth grow while oya_audit_chain_merkle_root_discrepancy_investigation_error_ratio is flat?
   |-- yes: downstream dependency, replay backlog, or queue-drain issue; choose mitigation branch B.
   |-- no: local regression, bad input, or policy/config drift; continue branch selection.
3. Does audit-chain show EVT_AUDIT_CHAIN_MERKLE_ROOT_DISCREPANCY_INVESTIGATION_INCIDENT gaps?
   |-- yes: do not close; run evidence replay before resolution.
   |-- no: mitigation can proceed after state is green.
4. Is customer, finance, security, or regulator impact confirmed?
   |-- yes: promote severity, open #inc-audit-chain, and notify compliance or security handoff.
   |-- no: keep internal incident and collect evidence.
```
- Branch A (confirmed chain integrity risk): use the matching mitigation block below and record `decision_branch=A` in `EVT_AUDIT_CHAIN_MERKLE_ROOT_DISCREPANCY_INVESTIGATION_INCIDENT`.
- Branch B (export-only degradation): use the matching mitigation block below and record `decision_branch=B` in `EVT_AUDIT_CHAIN_MERKLE_ROOT_DISCREPANCY_INVESTIGATION_INCIDENT`.
- Branch C (HSM or signer dependency degraded): use the matching mitigation block below and record `decision_branch=C` in `EVT_AUDIT_CHAIN_MERKLE_ROOT_DISCREPANCY_INVESTIGATION_INCIDENT`.
- Branch D (regulator-visible evidence gap): use the matching mitigation block below and record `decision_branch=D` in `EVT_AUDIT_CHAIN_MERKLE_ROOT_DISCREPANCY_INVESTIGATION_INCIDENT`.

## Mitigation Steps
1. Acknowledge page: `pd incident ack --service audit-chain --incident $INCIDENT_ID`.
2. Create bridge: `oya incident bridge create --incident $INCIDENT_ID --channel #inc-audit-chain --severity sev1`.
3. Freeze risky automation: `oya flags set oya.audit-chain.merkle_root_discrepancy_investigation.incident_hold=true --cell $CELL --tenant $TENANT --reason $INCIDENT_ID`.
4. Enable circuit breaker: `oya ops breaker open audit-chain-merkle-root-discrepancy-investigation-circuit-breaker --cell $CELL --tenant $TENANT --ttl 30m --reason $INCIDENT_ID`.
5. Reduce blast radius: `kubectl -n audit-chain scale deploy/audit-chain-merkle-root-discrepancy-investigation-worker --replicas=1`.
6. Protect tenant boundary: `oya tenancy quarantine --tenant $TENANT --reason audit-chain-merkle-root-discrepancy-investigation --ttl 60m`.
7. Pause promotion: incident hold PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
8. Drain queue safely: `oya ops audit-chain merkle-root-discrepancy-investigation drain --cell $CELL --tenant $TENANT --max-items 500 --dry-run`.
9. Execute bounded drain: `oya ops audit-chain merkle-root-discrepancy-investigation drain --cell $CELL --tenant $TENANT --max-items 500 --confirm $INCIDENT_ID`.
10. Replay missing audit events: `oya audit-chain replay --event-class EVT_AUDIT_CHAIN_MERKLE_ROOT_DISCREPANCY_INVESTIGATION_INCIDENT --incident $INCIDENT_ID --from evidence/incidents/$INCIDENT_ID.json`.
11. Rollback last deploy if causal: `kubectl -n audit-chain rollout undo deploy/audit-chain-merkle-root-discrepancy-investigation-worker`.
12. Raise HPA cap if saturation is proven: `kubectl -n audit-chain patch hpa audit-chain-merkle-root-discrepancy-investigation-worker --type merge -p '{"spec":{"maxReplicas":12}}'`.
13. Throttle hot tenant: `oya ops rate-limit set --tenant $TENANT --surface audit-chain.merkle-root-discrepancy-investigation --rps 25 --ttl 30m`.
14. Block abusive principal when relevant: `oya identity principal suspend --principal suspected-abuse --tenant $TENANT --reason $INCIDENT_ID`.
15. Protect evidence: `oya evidence freeze --incident $INCIDENT_ID --paths audit/runbooks/merkle-root-discrepancy-investigation.md,evidence/incidents/$INCIDENT_ID.json`.
16. Notify service owners: `oya notify service-owner --microservice audit-chain --incident $INCIDENT_ID --channel #inc-audit-chain`.
17. Open external vendor ticket: `oya vendor ticket open --vendor "Thales Luna HSM support" --incident $INCIDENT_ID --summary audit-chain-merkle-root-discrepancy-investigation`.
18. Confirm breaker effect: `oya ops breaker status audit-chain-merkle-root-discrepancy-investigation-circuit-breaker --cell $CELL --tenant $TENANT --expect open`.
19. Confirm user impact reduced: `curl -s https://audit-chain.internal.oyatie.dev/v1/audit-chain/merkle-root-discrepancy-investigation/incident-handoff/health -H "x-oya-tenant: $TENANT"`.
20. Emit mitigation audit: `oya audit-chain emit --event-class EVT_AUDIT_CHAIN_MERKLE_ROOT_DISCREPANCY_INVESTIGATION_INCIDENT --incident $INCIDENT_ID --field mitigation=active --field runbook=merkle-root-discrepancy-investigation`.

### Mitigation Branch Guidance
- Branch A: confirmed chain integrity risk.
  - Required action: keep `audit-chain-merkle-root-discrepancy-investigation-circuit-breaker` open until `oya_audit_chain_merkle_root_discrepancy_investigation_error_ratio` is below 0.005 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/audit-chain-substrate/merkle-root-discrepancy-investigation?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=117` to the incident.
  - Required audit: emit `EVT_AUDIT_CHAIN_MERKLE_ROOT_DISCREPANCY_INVESTIGATION_INCIDENT` with `branch=A`, `operator_id`, and `evidence_hash`.
- Branch B: export-only degradation.
  - Required action: keep `audit-chain-merkle-root-discrepancy-investigation-circuit-breaker` open until `oya_audit_chain_merkle_root_discrepancy_investigation_error_ratio` is below 0.01 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/audit-chain-substrate/merkle-root-discrepancy-investigation?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=118` to the incident.
  - Required audit: emit `EVT_AUDIT_CHAIN_MERKLE_ROOT_DISCREPANCY_INVESTIGATION_INCIDENT` with `branch=B`, `operator_id`, and `evidence_hash`.
- Branch C: HSM or signer dependency degraded.
  - Required action: keep `audit-chain-merkle-root-discrepancy-investigation-circuit-breaker` open until `oya_audit_chain_merkle_root_discrepancy_investigation_error_ratio` is below 0.005 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/audit-chain-substrate/merkle-root-discrepancy-investigation?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=119` to the incident.
  - Required audit: emit `EVT_AUDIT_CHAIN_MERKLE_ROOT_DISCREPANCY_INVESTIGATION_INCIDENT` with `branch=C`, `operator_id`, and `evidence_hash`.
- Branch D: regulator-visible evidence gap.
  - Required action: keep `audit-chain-merkle-root-discrepancy-investigation-circuit-breaker` open until `oya_audit_chain_merkle_root_discrepancy_investigation_error_ratio` is below 0.01 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/audit-chain-substrate/merkle-root-discrepancy-investigation?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=120` to the incident.
  - Required audit: emit `EVT_AUDIT_CHAIN_MERKLE_ROOT_DISCREPANCY_INVESTIGATION_INCIDENT` with `branch=D`, `operator_id`, and `evidence_hash`.

## Resolution Steps
1. Identify code owner path: `rg "merkle_root_discrepancy_investigation|AuditChainMerkleRootDiscrepancyInvestigationCritical|audit_chain.merkle_root_discrepancy_investigation.incident_state" crates microservices/audit-chain -g "!audit/runbooks/**"`.
2. Patch domain invariant: `edit oya-audit-chain-domain where merkle_root_discrepancy_investigation state transition is validated`.
3. Patch API guard: `edit audit/contracts/openapi/audit-chain.yaml if the failing path is north-south or async handoff`.
4. Patch policy: `edit microservices/audit-chain/policy/seal-integrity.md with explicit deny/permit branch and tenant/cell scope`.
5. Patch runtime config: `edit audit/iac/kustomize/base/kustomization.yaml if deploy/config drift caused the incident`.
6. Add regression test: `cargo test -p oya-audit-chain-domain merkle_root_discrepancy_investigation_incident_regression -- --nocapture`.
7. Add gate evidence: `cargo run -p oya-dev-cli -- gate validate audit-chain-merkle-root-discrepancy-investigation --fixture incident-merkle-root-discrepancy-investigation.json`.
8. Add SLO assertion: `update audit/observability/slos/chain-of-custody-integrity-correctness.openslo.yaml with alert AuditChainMerkleRootDiscrepancyInvestigationCritical when this was a missing alert`.
9. Add dashboard panel: `update audit/dashboards/emission-rate.json with oya_audit_chain_merkle_root_discrepancy_investigation_error_ratio, oya_audit_chain_merkle_root_discrepancy_investigation_lag_seconds, and oya_audit_chain_merkle_root_discrepancy_investigation_queue_depth`.
10. Rebuild affected crate: `cargo check -p oya-audit-chain-domain --all-targets`.
11. Run targeted tests: `cargo test -p oya-audit-chain-domain --all-features`.
12. Run policy validation: `cargo run -p oya-dev-cli -- gate validate audit-chain-policy --microservice audit-chain`.
13. Deploy canary: `oya deploy canary --microservice audit-chain --component audit-chain-merkle-root-discrepancy-investigation-worker --cell $CELL --weight 1`.
14. Watch burn rate: `oya ops watch --metric oya_audit_chain_merkle_root_discrepancy_investigation_error_ratio --threshold 0.005 --window 30m --cell $CELL`.
15. Close circuit breaker: `oya ops breaker close audit-chain-merkle-root-discrepancy-investigation-circuit-breaker --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
16. Unfreeze automation: `oya flags set oya.audit-chain.merkle_root_discrepancy_investigation.incident_hold=false --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
17. Resume promotion: recovery PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
18. Seal resolution audit: `oya audit-chain emit --event-class EVT_AUDIT_CHAIN_MERKLE_ROOT_DISCREPANCY_INVESTIGATION_INCIDENT --incident $INCIDENT_ID --field resolution=complete --field runbook=merkle-root-discrepancy-investigation`.
19. Verify seal: `oya audit-chain verify --event-class EVT_AUDIT_CHAIN_MERKLE_ROOT_DISCREPANCY_INVESTIGATION_INCIDENT --incident $INCIDENT_ID`.
20. Attach final evidence: `oya evidence attach --incident $INCIDENT_ID --file evidence/incidents/$INCIDENT_ID.json --kind final-resolution`.

### Code Paths To Inspect First
- `oya-audit-chain-domain`: inspect for `merkle_root_discrepancy_investigation` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `oya-audit-chain-usecase`: inspect for `merkle_root_discrepancy_investigation` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `oya-audit-chain-file-adapter`: inspect for `merkle_root_discrepancy_investigation` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `oya-shared-audit-chain-client-kernel`: inspect for `merkle_root_discrepancy_investigation` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `audit/contracts/openapi/audit-chain.yaml`: verify request/response or event contract only when incident evidence points there.
- `audit/contracts/asyncapi/audit-events.yaml`: verify request/response or event contract only when incident evidence points there.
- `audit/contracts/proto/audit-chain.proto`: verify request/response or event contract only when incident evidence points there.
- `audit/dashboards/emission-rate.json`: verify panel coverage for `oya_audit_chain_merkle_root_discrepancy_investigation_error_ratio`, `oya_audit_chain_merkle_root_discrepancy_investigation_lag_seconds`, and `oya_audit_chain_merkle_root_discrepancy_investigation_merkle_gap_total`.
- `audit/observability/slos/`: verify alert vocabulary and threshold alignment before changing runtime thresholds.
- `audit/policy/`: verify policy branch ownership before relaxing deny rules or emergency bypasses.

## Verification Checklist
- `AuditChainMerkleRootDiscrepancyInvestigationCritical` and `AuditChainMerkleRootDiscrepancyInvestigationSloBurn` are both resolved in Alertmanager for 30 minutes.
- `oya_audit_chain_merkle_root_discrepancy_investigation_error_ratio < 0.005` for 3 consecutive 10 minute windows.
- `oya_audit_chain_merkle_root_discrepancy_investigation_lag_seconds < 120` for all production cells.
- `oya_audit_chain_merkle_root_discrepancy_investigation_queue_depth` is draining and not growing for the affected tenant.
- Service-specific signal `oya_audit_chain_merkle_root_discrepancy_investigation_merkle_gap_total` is below the threshold documented in `audit/observability/slos/chain-of-custody-integrity-correctness.openslo.yaml`.
- dashboard `https://grafana.dev.oyatie.internal/d/audit-chain-substrate/merkle-root-discrepancy-investigation?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=116` shows green panels for the affected cell.
- audit-chain query for `EVT_AUDIT_CHAIN_MERKLE_ROOT_DISCREPANCY_INVESTIGATION_INCIDENT` returns mitigation and resolution events.
- circuit breaker `audit-chain-merkle-root-discrepancy-investigation-circuit-breaker` is closed after rollback window.
- feature flag `oya.audit-chain.merkle_root_discrepancy_investigation.incident_hold` is false for the affected tenant unless long-term hold is approved.
- runbook invocation evidence is attached to `evidence/incidents/$INCIDENT_ID.json`.
- service owner acknowledged final handoff in `#inc-audit-chain`.

## Postmortem Template
Use this exact skeleton for the incident document. The field names are intentionally stable for ADR-0263 audit emission extraction.
```markdown
---
doc_class: IncidentPostmortem
runbook_id: audit-chain-merkle-root-discrepancy-investigation
microservice: audit-chain
event_class: EVT_AUDIT_CHAIN_MERKLE_ROOT_DISCREPANCY_INVESTIGATION_INCIDENT
incident_id: <INC-...>
severity: sev1
status: draft
detected_at: <UTC>
mitigated_at: <UTC>
resolved_at: <UTC>
commander: <handle>
evidence_hash: <sha256>
---

# Merkle Root Discrepancy Investigation postmortem

## Summary
- What happened in audit-chain/merkle-root-discrepancy-investigation.
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
- Emit EVT_AUDIT_CHAIN_MERKLE_ROOT_DISCREPANCY_INVESTIGATION_INCIDENT with incident_id, tenant_id, cell_id, principal_id, decision_id, evidence_hash, operator_id, runbook_id.
- Attach dashboard snapshot URLs and command transcripts.
- Seal mitigation and resolution events before closure.

## Corrective Actions
- Action, owner, due date, validation command, linked issue.
```

## Escalation Path
- Primary on-call: PagerDuty oya-audit-chain-primary; compliance-evidence secondary.
- Incident SLA: ack 3m for sev0/sev1, 10m for sev2, 30m for sev3; status update every 10m until `AuditChainMerkleRootDiscrepancyInvestigationCritical` clears.
- Incident commander: first responder from axis-audit-chain + ops-sre-reliability + ops-security; transfer only by explicit message in `#inc-audit-chain`.
- Security escalation: page `ops-security-primary` immediately for sev0, credential, cross-tenant, fraud, or audit-seal symptoms.
- Compliance escalation: page `dpo-office-duty` when tenant data, regulator evidence, money movement, or breach-clock symptoms are present.
- Architecture escalation: page `council-architecture-reviewer` before manual bypass, policy rollback, or invariant relaxation.
- External vendors: Thales Luna HSM support; DigiCert timestamp authority support; Oracle PostgreSQL support. Open a ticket once local dependency health is proven and vendor dependency remains suspect.
- Customer communications: use status page component `oyatie-audit-chain-merkle-root-discrepancy-investigation` and keep private details in the incident channel.
- Regulatory clock: if tenant data, financial correctness, or evidence integrity is possibly affected, start the compliance 72h assessment timer even if exposure is unconfirmed.
- Executive notice: sev0 or fleet-wide sev1 goes to `#exec-incident-readout` within 30 minutes.

## Cross-µservice Coordination
- Notify `observability`: `oya incident handoff --target observability --source audit-chain --runbook merkle-root-discrepancy-investigation --incident $INCIDENT_ID --severity sev1 --branch A`; expect `202 accepted`.
- Require `observability` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `compliance`: `oya incident handoff --target compliance --source audit-chain --runbook merkle-root-discrepancy-investigation --incident $INCIDENT_ID --severity sev1 --branch B`; expect `202 accepted`.
- Require `compliance` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `governance`: `oya incident handoff --target governance --source audit-chain --runbook merkle-root-discrepancy-investigation --incident $INCIDENT_ID --severity sev1 --branch C`; expect `202 accepted`.
- Require `governance` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `tenancy`: `oya incident handoff --target tenancy --source audit-chain --runbook merkle-root-discrepancy-investigation --incident $INCIDENT_ID --severity sev1 --branch D`; expect `202 accepted`.
- Require `tenancy` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Observability handoff API: `oya incident handoff --target observability --source audit-chain --runbook merkle-root-discrepancy-investigation --incident $INCIDENT_ID`.
- Governance handoff API: `oya incident handoff --target governance --source audit-chain --runbook merkle-root-discrepancy-investigation --incident $INCIDENT_ID`.
- Compliance handoff API: `oya incident handoff --target compliance --source audit-chain --runbook merkle-root-discrepancy-investigation --incident $INCIDENT_ID`.
- Audit-chain handoff API: `oya incident handoff --target audit-chain --source audit-chain --runbook merkle-root-discrepancy-investigation --incident $INCIDENT_ID`.
- Tenancy handoff API: `oya incident handoff --target tenancy --source audit-chain --runbook merkle-root-discrepancy-investigation --incident $INCIDENT_ID`.

## Handoff Notes
- Do not hand off with only the alert name; include `oya_audit_chain_merkle_root_discrepancy_investigation_error_ratio`, `oya_audit_chain_merkle_root_discrepancy_investigation_lag_seconds`, `oya_audit_chain_merkle_root_discrepancy_investigation_queue_depth`, `oya_audit_chain_merkle_root_discrepancy_investigation_merkle_gap_total`, current breaker state, and audit seal status.
- Keep `audit-chain-merkle-root-discrepancy-investigation-circuit-breaker` owner as axis-audit-chain + ops-sre-reliability + ops-security until the receiving service explicitly accepts.
- If another runbook owns the downstream fix, link this incident as upstream and keep this runbook open until downstream verification returns green.
- Close only after `EVT_AUDIT_CHAIN_MERKLE_ROOT_DISCREPANCY_INVESTIGATION_INCIDENT` has a sealed resolution row and every coordination endpoint above has either accepted or explicitly declined scope.

## Sources Checked During This Substance Pass
- `audit/dashboards/` for dashboard names and operational panels: emission-rate.json, seal-latency.json, verification-failure-rate.json.
- `audit/observability/slos/` for OpenSLO alert vocabulary and threshold alignment: chain-of-custody-integrity-correctness.openslo.yaml, evidence-export-freshness.openslo.yaml, merkle-chain-verification-latency.openslo.yaml.
- `audit/policy/` for named policy and authorization surfaces: seal-integrity.md, auditor-scope.cedar, tenant-scope.cedar.
- `audit/contracts/` for API, AsyncAPI, proto, and adapter surfaces: contracts/openapi/audit-chain.yaml, contracts/asyncapi/audit-events.yaml, contracts/proto/audit-chain.proto.
- `audit/catalog/` for component and owner vocabulary; existing runbook topic `merkle-root-discrepancy-investigation` was preserved as the scenario anchor.
