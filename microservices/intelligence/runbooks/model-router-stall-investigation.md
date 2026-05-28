---
doc_class: Runbook
title: Model Router Stall Investigation
status: Accepted
date: 2026-05-20
microservice: intelligence
severity: sev1
audience: sre, ai-safety-engineer, intelligence-operator
owner_team: axis-intelligence + ops-sre-reliability + ai-safety
doc_status: published
---

# Runbook: Model Router Stall Investigation

## Operator Contract
- Runbook id: intelligence-model-router-stall-investigation.
- Primary namespace: `intelligence`.
- Owning rotation: PagerDuty `oya-intelligence-primary`.
- Safety secondary: PagerDuty `oya-ai-safety-primary`.
- Incident channel: `#inc-intelligence`.
- Customer channel: `#support-ai-dispatch-impact`.
- Protected surface: model-routing kernel, provider adapters, credential resolver, guardrail stack, eval gate, audit tap.
- Provider surfaces: OpenAI, Anthropic, Google Vertex, Bedrock, BYOK credential paths.
- Safety invariant: do not bypass guardrails or audit tap to drain router backlog.
- Residency invariant: do not route across pack or region boundary without Cedar permit.
- Stop condition: dispatch queue drains, router decisions are current, provider fallback is bounded, and audit events seal.
- Evidence event: `EVT_INTELLIGENCE_MODEL_ROUTER_STALL_INCIDENT`.
- Handoff API: `https://intelligence.internal.oyatie.dev/v1/intelligence/model-router/incidents/$INCIDENT_ID/handoff`.
- Primary dashboard: `https://grafana.dev.oyatie.internal/d/intelligence-substrate/model-router?orgId=1&var-cell=prod-us-east-1`.
- Provider dashboard: `https://grafana.dev.oyatie.internal/d/intelligence-substrate/provider-latency?orgId=1&var-provider=all`.
- Loki query: `{namespace="intelligence",runbook="model-router-stall-investigation"}`.
- Canonical catalog: `microservices/intelligence/catalog/oya-intelligence-model-routing-kernel.yaml`.
- Related dashboard: `microservices/intelligence/dashboards/provider-latency-heatmap.json`.
- Related SLO: `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`.
- Related policy: `microservices/intelligence/policy/provider-routing.cedar`.

## Trigger Conditions
- Alert `IntelligenceModelRouterStallCritical` fires.
- Alert `IntelligenceDispatchQueueSloBurn` fires for 10 minutes.
- Alert `IntelligenceProviderFallbackLoopDetected` fires.
- Alert `IntelligenceRouterDecisionAgeHigh` fires.
- Alert `IntelligenceAuditTapBackpressure` fires.
- Metric `oya_intelligence_model_router_queue_depth` exceeds 5000.
- Metric `oya_intelligence_model_router_decision_age_seconds` exceeds 300.
- Metric `oya_intelligence_dispatch_latency_p99_seconds` exceeds SLO.
- Metric `oya_intelligence_provider_fallback_loop_total` increases.
- Metric `oya_intelligence_provider_timeout_ratio` exceeds 0.02.
- Metric `oya_intelligence_credential_resolution_lag_seconds` exceeds 120.
- Metric `oya_intelligence_guardrail_eval_lag_seconds` exceeds 120.
- Metric `oya_intelligence_audit_tap_backpressure_seconds` exceeds 60.
- Provider outage runbook is already active for one provider.
- BYOK rotation is active for affected tenants.
- Tenant cost cap or residency policy denies all candidate routes.
- Eval gate blocks new model route promotion.
- Dispatch API returns `RouterUnavailable`.
- Customer support tags case `intelligence.model-router-stall`.
- Audit-chain lacks `intelligence.model_route.decision` after dispatch intake.

## Symptoms
- Dispatch requests remain queued before provider call.
- Provider adapters are healthy but router emits no decision.
- Router repeatedly chooses provider fallback then requeues.
- `router_state=stalled` appears in dispatch logs.
- `candidate_provider_count=0` appears for one tenant.
- `fallback_loop=true` appears for one dispatch id.
- `credential_resolver_wait=true` appears before route decision.
- `guardrail_eval_wait=true` appears before route decision.
- `audit_tap_wait=true` appears before response return.
- Residency policy refuses every provider candidate.
- Tenant cost cap denies all non-local providers.
- Provider catalog version differs between route workers.
- Model deprecation event removes last allowed model for a pack.
- Queue depth rises while provider 5xx and 429 are flat.
- Dispatch latency burns but provider latency dashboard is green.
- First-token latency is high only because dispatch never reaches provider.
- User-facing AI drafts, RAG answers, and summarization stall.
- Safety impact is high if operators request guardrail bypass.
- Customer impact is broad if router stall is fleet-wide.
- Severity rises to Sev0 if audit tap is bypassed or unsafe route is chosen.

## Diagnostic Steps
1. Set scope: `export INCIDENT_ID=INC-intelligence-router-stall-$(date -u +%Y%m%dT%H%M%SZ)`.
2. Set defaults: `export CELL=prod-us-east-1; export TENANT=synthetic-canary; export PACK=canonical-base`.
3. Acknowledge page: `pd incident ack --service intelligence --incident $INCIDENT_ID`.
4. Create bridge: `oya incident bridge create --incident $INCIDENT_ID --channel #inc-intelligence --severity sev1`.
5. Query active alerts: `curl -s https://alertmanager.dev.oyatie.internal/api/v2/alerts | jq '.[] | select(.labels.service=="intelligence")'`.
6. Query router queue: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_intelligence_model_router_queue_depth{cell="'$CELL'"}'`.
7. Query decision age: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_intelligence_model_router_decision_age_seconds{tenant_id="'$TENANT'"}'`.
8. Query dispatch latency: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_intelligence_dispatch_latency_p99_seconds{cell="'$CELL'"}'`.
9. Query fallback loops: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=rate(oya_intelligence_provider_fallback_loop_total[5m])'`.
10. Query audit backpressure: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_intelligence_audit_tap_backpressure_seconds{cell="'$CELL'"}'`.
11. Open router dashboard: `open "https://grafana.dev.oyatie.internal/d/intelligence-substrate/model-router?orgId=1&var-cell=$CELL&var-tenant=$TENANT"`.
12. Open provider dashboard: `open "https://grafana.dev.oyatie.internal/d/intelligence-substrate/provider-latency?orgId=1&var-cell=$CELL&var-provider=all"`.
13. Read router logs: `kubectl -n intelligence logs deploy/intelligence-model-router --since=30m | rg "router_state|fallback_loop|candidate_provider|decision"`.
14. Read dispatch logs: `kubectl -n intelligence logs deploy/intelligence-dispatch-api --since=30m | rg "RouterUnavailable|model_route|audit_tap|guardrail"`.
15. Check rollout: `kubectl -n intelligence rollout status deploy/intelligence-model-router --timeout=60s`.
16. List router pods: `kubectl -n intelligence get pods -l app=model-router -o wide`.
17. Inspect router state: `oya ops intelligence model-router status --tenant $TENANT --cell $CELL --output json`.
18. Explain route: `oya ops intelligence model-router explain --tenant $TENANT --pack $PACK --task assist-draft --output yaml`.
19. Check provider catalog: `oya ops intelligence provider-catalog status --tenant $TENANT --pack $PACK --output json`.
20. Check provider health: `oya ops intelligence provider health --tenant $TENANT --provider all --cell $CELL --output table`.
21. Check provider rate limit: `oya ops intelligence provider quota --tenant $TENANT --provider all --cell $CELL --output table`.
22. Check credential resolver: `oya ops intelligence credential-resolver status --tenant $TENANT --provider all --cell $CELL --output json`.
23. Check BYOK rotation: `oya ops intelligence byok rotation status --tenant $TENANT --cell $CELL --output json`.
24. Check guardrail gate: `oya ops intelligence guardrails status --tenant $TENANT --pack $PACK --output json`.
25. Check eval gate: `oya ops intelligence eval-gate status --tenant $TENANT --pack $PACK --output json`.
26. Check audit tap: `oya ops intelligence audit-tap status --tenant $TENANT --cell $CELL --output json`.
27. Check residency policy: `oya ops intelligence residency route-check --tenant $TENANT --pack $PACK --output json`.
28. Check cost cap: `oya ops intelligence cost-cap status --tenant $TENANT --period current --output json`.
29. Query route decisions: `oya audit-chain query --event-class intelligence.model_route.decision --tenant $TENANT --since 30m`.
30. Query dispatch intake: `oya audit-chain query --event-class intelligence.dispatch.accepted --tenant $TENANT --since 30m`.
31. Query refusal events: `oya audit-chain query --event-class intelligence.refusal.decision --tenant $TENANT --since 30m`.
32. Check active provider runbooks: `oya incident list --service intelligence --tag provider-outage --state active`.
33. Snapshot evidence: `oya evidence snapshot --incident $INCIDENT_ID --microservice intelligence --runbook model-router-stall-investigation --output evidence/incidents/$INCIDENT_ID.json`.
34. Export route state: `oya ops intelligence model-router export --tenant $TENANT --cell $CELL --output evidence/incidents/$INCIDENT_ID-router.json`.
35. Export provider matrix: `oya ops intelligence provider matrix --tenant $TENANT --pack $PACK --output evidence/incidents/$INCIDENT_ID-provider-matrix.json`.

### Diagnostic Decision Tree
```text
1. Is audit tap backpressure blocking responses?
   |-- yes: keep fail-closed and coordinate audit-chain before routing changes.
   |-- no: continue router triage.
2. Are provider adapters healthy?
   |-- no: invoke provider outage or rate-limit runbook.
   |-- yes: inspect candidate generation.
3. Is candidate provider count zero?
   |-- yes: inspect residency, cost cap, credential, and catalog policies.
   |-- no: inspect fallback loop and route worker health.
4. Is BYOK credential resolution lagging?
   |-- yes: invoke credential resolver or BYOK rotation runbook.
   |-- no: inspect guardrail and eval gates.
5. Does route explain produce safe candidate after mitigation?
   |-- yes: drain router queue.
   |-- no: keep incident open and page AI safety.
```

## Mitigation
1. Hold route policy deploys: incident hold PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
2. Keep guardrails enforced: `oya flags set oya.intelligence.guardrails.bypass=false --tenant $TENANT --cell $CELL --reason $INCIDENT_ID`.
3. Keep audit tap enforced: `oya flags set oya.intelligence.audit_tap.required=true --tenant $TENANT --cell $CELL --reason $INCIDENT_ID`.
4. Open router breaker: `oya ops breaker open intelligence-model-router --cell $CELL --tenant $TENANT --ttl 30m --reason $INCIDENT_ID`.
5. Disable unsafe provider candidate: `oya ops intelligence provider disable --tenant $TENANT --provider <provider> --reason $INCIDENT_ID --ttl 30m`.
6. Pin known-good provider catalog: `oya ops intelligence provider-catalog pin --tenant $TENANT --version previous-stable --reason $INCIDENT_ID`.
7. Refresh credential handles: `oya ops intelligence credential-resolver refresh --tenant $TENANT --provider all --confirm $INCIDENT_ID`.
8. Refresh route cache: `oya ops intelligence model-router cache invalidate --tenant $TENANT --cell $CELL --reason $INCIDENT_ID`.
9. Drain queue dry-run: `oya ops intelligence model-router drain --tenant $TENANT --cell $CELL --limit 200 --dry-run`.
10. Drain queue confirmed: `oya ops intelligence model-router drain --tenant $TENANT --cell $CELL --limit 200 --confirm $INCIDENT_ID`.
11. Roll back causal deploy: `kubectl -n intelligence rollout undo deploy/intelligence-model-router`.
12. Scale router workers if local CPU saturation is proven: `kubectl -n intelligence scale deploy/intelligence-model-router --replicas=8`.
13. Throttle hot tenant: `oya ops rate-limit set --tenant $TENANT --surface intelligence.dispatch --rps 20 --ttl 30m`.
14. Notify support: `oya notify support --incident $INCIDENT_ID --template intelligence-router-stall`.
15. Notify tenant admin when dispatch is customer-visible: `oya notify tenant-admin --tenant $TENANT --incident $INCIDENT_ID --template ai-dispatch-delayed`.
16. Notify AI safety: `oya notify ai-safety --incident $INCIDENT_ID --category model-routing`.
17. Emit mitigation audit: `oya audit-chain emit --event-class EVT_INTELLIGENCE_MODEL_ROUTER_STALL_INCIDENT --incident $INCIDENT_ID --field mitigation=router-queue-contained`.
18. Keep no-route decisions explicit rather than unsafe fallback.
19. Keep provider route changes tenant-scoped.
20. Keep dashboard snapshots in evidence.

## Resolution
1. Patch candidate generation if all candidates were incorrectly filtered.
2. Patch provider catalog projection if model deprecation removed valid successors.
3. Patch fallback loop detection if routes requeued indefinitely.
4. Patch credential resolver timeout if BYOK path blocked candidates.
5. Patch residency or cost cap explainability if all routes denied silently.
6. Patch audit tap backpressure handling if response waits were unbounded.
7. Add regression fixture for zero candidate providers.
8. Add regression fixture for fallback loop.
9. Run model routing tests: `cargo test -p oya-intelligence-route-policy-kernel model_router -- --nocapture`.
10. Run eval tests: `cargo test -p oya-governance-eval-domain eval_gate -- --nocapture`.
11. Run production gate: `cargo run -p oya-dev-cli -- gate validate intelligence-model-router --production-snapshot --cell $CELL`.
12. Verify route explain: `oya ops intelligence model-router explain --tenant $TENANT --pack $PACK --task assist-draft --expect safe-candidate`.
13. Close breaker: `oya ops breaker close intelligence-model-router --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
14. Unhold deploys: recovery PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
15. Seal audit: `oya audit-chain emit --event-class EVT_INTELLIGENCE_MODEL_ROUTER_STALL_INCIDENT --incident $INCIDENT_ID --field resolution=complete`.

## Verification Checklist
- `IntelligenceModelRouterStallCritical` is green.
- `oya_intelligence_model_router_queue_depth` returns to baseline.
- `oya_intelligence_model_router_decision_age_seconds` is below 60.
- Dispatch p99 latency returns to SLO.
- Provider fallback loop metric is flat.
- Audit tap backpressure is below 10 seconds.
- Route explain returns at least one safe candidate.
- Audit-chain contains dispatch intake and route decision events.
- Guardrail bypass flag is false.
- Support reports no new `intelligence.model-router-stall` cases.

## Postmortem Template
```markdown
---
doc_class: IncidentPostmortem
runbook_id: intelligence-model-router-stall-investigation
microservice: intelligence
event_class: EVT_INTELLIGENCE_MODEL_ROUTER_STALL_INCIDENT
incident_id: <INC-...>
severity: sev1
status: draft
detected_at: <UTC>
mitigated_at: <UTC>
resolved_at: <UTC>
commander: <handle>
evidence_hash: <sha256>
---

# Model Router Stall Investigation postmortem

## Summary
- Which tenant, pack, task, provider matrix, and route policy stalled.
- Whether stall was provider, credential, guardrail, audit, residency, or cost cap.
- Whether unsafe fallback was requested or avoided.

## Timeline
- Queue growth detected:
- Router breaker opened:
- Safe candidate restored:
- Queue drained:
- Audit sealed:

## Customer Impact
- Dispatches delayed:
- Tasks affected:
- Tenants affected:
- Safety posture:

## Root Cause
- Candidate generation:
- Provider catalog:
- Credential resolver:
- Guardrail or eval:
- Audit tap:

## Corrective Actions
- Owner:
- Due date:
- Regression test:
- Dashboard or alert update:
```

## Escalation Path
- Page `oya-intelligence-primary` for router stall.
- Page `oya-ai-safety-primary` if guardrail, eval, or unsafe fallback is implicated.
- Page provider owner when adapter health is failing.
- Page `oya-audit-chain-primary` when audit tap blocks or events are missing.
- Page `oya-cloud-kms-primary` when BYOK credential path blocks routing.
- Notify `#inc-intelligence` with tenant, pack, and provider matrix.
- Notify `#support-ai-dispatch-impact` before tenant communication.
- Notify `#compliance-review` when residency or regulated pack routing is involved.
- Escalate to executive incident commander if fleet-wide dispatch stalls.
- Keep all unsafe fallback requests denied unless safety commander approves a documented alternative.

## Cross-µservice Coordination
- `audit-chain`: seal dispatch, route decision, mitigation, and resolution events.
- `cloud-kms`: verify BYOK and credential resolver key paths.
- `cloud-iam`: verify tenant, workload, and operator authorization for dispatch.
- `cloud-network`: verify provider egress and regional routing.
- `cloud-billing`: verify tenant cost caps and provider spend constraints.
- `tenancy`: verify pack, residency, and tenant tier.
- `support`: manage customer-visible AI dispatch cases.
- `ai-safety`: own guardrail and unsafe fallback decisions.
- `observability`: attach router and provider dashboards.
- `foundry`: pause route policy or provider catalog deploys.
- `workflow-engine`: pause workflows waiting on AI dispatch.
- `comms-email`: send approved delay and all-clear notices.

## Runbook Maintenance
- Add every new router stall reason to Symptoms.
- Keep provider names aligned with catalog.
- Keep unsafe fallback prohibition explicit.
- Review this runbook after every provider catalog change.
- Add new dispatch tasks to route explain examples.
