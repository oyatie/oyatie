---
doc_class: Runbook
title: Prompt Fence Bypass Detection
status: Accepted
date: 2026-05-20
microservice: intelligence
severity: sev0
audience: ai-safety-engineer, security-engineer, sre
owner_team: axis-intelligence + ai-safety + security-governance
doc_status: published
---

# Runbook: Prompt Fence Bypass Detection

## Operator Contract
- Runbook id: intelligence-prompt-fence-bypass-detection.
- Primary namespace: `intelligence`.
- Owning rotation: PagerDuty `oya-ai-safety-primary`.
- Intelligence secondary: PagerDuty `oya-intelligence-primary`.
- Incident channel: `#inc-ai-safety`.
- Customer channel: `#support-ai-safety`.
- Protected surface: prompt fence, system prompt boundary, tool-call policies, RAG citations, refusal baseline, audit tap.
- Safety invariant: suspicious output is refused before it reaches customer-visible surfaces.
- Evidence invariant: preserve prompt hash, model route, guardrail decision, and output hash; do not paste raw sensitive prompt text into chat.
- Privacy invariant: raw prompt and output exports stay in evidence storage only.
- Stop condition: bypass pattern is blocked, affected outputs are contained, eval regression exists, and audit-chain is sealed.
- Evidence event: `EVT_INTELLIGENCE_PROMPT_FENCE_BYPASS_DETECTION_INCIDENT`.
- Handoff API: `https://intelligence.internal.oyatie.dev/v1/intelligence/prompt-fence/incidents/$INCIDENT_ID/handoff`.
- Primary dashboard: `https://grafana.dev.oyatie.internal/d/intelligence-substrate/prompt-fence?orgId=1&var-cell=prod-us-east-1`.
- Safety dashboard: `https://grafana.dev.oyatie.internal/d/intelligence-substrate/refusal-rate-by-pack?orgId=1&var-pack=canonical-base`.
- Loki query: `{namespace="intelligence",runbook="prompt-fence-bypass-detection"}`.
- Canonical policy: `microservices/intelligence/policy/refusal-baseline.cedar`.
- Related dashboard: `microservices/intelligence/dashboards/prompt-injection-detection.md`.
- Related SLO: `microservices/intelligence/slos/refusal-false-negative-rate.openslo.yaml`.

## Trigger Conditions
- Alert `IntelligencePromptFenceBypassDetectedCritical` fires.
- Alert `IntelligenceRefusalFalseNegativeBurn` fires.
- Alert `IntelligenceToolPolicyBypassDetected` fires.
- Alert `IntelligencePromptBoundaryLeakDetected` fires.
- Alert `IntelligenceRagCitationPolicyBypass` fires.
- Metric `oya_intelligence_prompt_fence_bypass_detected_total` is non-zero.
- Metric `oya_intelligence_refusal_false_negative_rate` exceeds SLO.
- Metric `oya_intelligence_tool_policy_bypass_total` increases.
- Metric `oya_intelligence_system_prompt_leak_score` exceeds threshold.
- Metric `oya_intelligence_guardrail_post_call_block_total` spikes.
- Metric `oya_intelligence_prompt_injection_detected_total` spikes.
- Metric `oya_intelligence_safety_eval_regression_total` increases.
- Customer reports model revealed policy, instructions, secrets, or forbidden tool path.
- Red-team canary detects successful jailbreak.
- RAG answer includes instruction override from corpus text.
- Tool call executes outside approved action set.
- Prompt fence hash differs from deployed policy bundle.
- Eval gate allows model route after safety score drop.
- Audit-chain lacks `intelligence.prompt_fence.checked` for dispatch.
- Existing prompt bypass attempt response runbook is active and detection confirms success.

## Symptoms
- Output contains system prompt language or internal policy names.
- Output references hidden tool names or credential paths.
- Output follows user instruction to ignore prior rules.
- Output cites RAG text as instruction instead of evidence.
- Guardrail pre-call allowed but post-call blocked.
- Guardrail post-call missed and customer report arrives.
- `prompt_fence_status=bypassed` appears in logs.
- `tool_policy_status=bypassed` appears in logs.
- `system_prompt_leak_score` is high for one model route.
- `prompt_fence_hash_mismatch=true` appears after deploy.
- One provider or model family dominates bypass detections.
- One tenant pack has elevated false-negative rate.
- BYOK model route bypasses platform default guardrail.
- RAG corpus chunk contains adversarial instruction.
- Tool-call policy lacks Cedar deny for a new tool.
- Safety eval canonicalen-set score dropped after prompt template update.
- Support case contains raw sensitive prompt or output and must be moved to evidence storage.
- Customer-visible impact is trust and safety, even for one output.
- Severity is Sev0 when bypass reached customer or tool execution.
- Severity is Sev1 when bypass was blocked before output.

## Diagnostic Steps
1. Set scope: `export INCIDENT_ID=INC-intelligence-prompt-fence-$(date -u +%Y%m%dT%H%M%SZ)`.
2. Set defaults: `export CELL=prod-us-east-1; export TENANT=synthetic-canary; export PACK=canonical-base`.
3. Acknowledge page: `pd incident ack --service ai-safety --incident $INCIDENT_ID`.
4. Create bridge: `oya incident bridge create --incident $INCIDENT_ID --channel #inc-ai-safety --severity sev0`.
5. Query active alerts: `curl -s https://alertmanager.dev.oyatie.internal/api/v2/alerts | jq '.[] | select(.labels.surface=="prompt-fence")'`.
6. Query bypass count: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=rate(oya_intelligence_prompt_fence_bypass_detected_total[5m])'`.
7. Query false-negative rate: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_intelligence_refusal_false_negative_rate{pack="'$PACK'"}'`.
8. Query tool bypass: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=rate(oya_intelligence_tool_policy_bypass_total[5m])'`.
9. Query leak score: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_intelligence_system_prompt_leak_score{tenant_id="'$TENANT'"}'`.
10. Query safety eval: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_intelligence_safety_eval_regression_total{pack="'$PACK'"}'`.
11. Open prompt fence dashboard: `open "https://grafana.dev.oyatie.internal/d/intelligence-substrate/prompt-fence?orgId=1&var-cell=$CELL&var-tenant=$TENANT"`.
12. Open refusal dashboard: `open "https://grafana.dev.oyatie.internal/d/intelligence-substrate/refusal-rate-by-pack?orgId=1&var-pack=$PACK"`.
13. Read guardrail logs: `kubectl -n intelligence logs deploy/intelligence-guardrails --since=60m | rg "prompt_fence|bypass|tool_policy|system_prompt"`.
14. Read dispatch logs: `kubectl -n intelligence logs deploy/intelligence-dispatch-api --since=60m | rg "guardrail|prompt_hash|output_hash|route_id"`.
15. Inspect event: `oya ops intelligence prompt-fence event get --tenant $TENANT --incident $INCIDENT_ID --output json`.
16. Export prompt hashes only: `oya ops intelligence prompt-fence export-hashes --tenant $TENANT --since 60m --output evidence/incidents/$INCIDENT_ID-hashes.json`.
17. Freeze raw evidence: `oya ops intelligence prompt-fence evidence-freeze --tenant $TENANT --incident $INCIDENT_ID`.
18. Check prompt fence bundle: `oya ops intelligence prompt-fence bundle status --tenant $TENANT --pack $PACK --output json`.
19. Check bundle digest: `oya ops intelligence prompt-fence bundle digest --tenant $TENANT --pack $PACK --output json`.
20. Check model route: `oya ops intelligence model-router explain --tenant $TENANT --pack $PACK --task suspected --output yaml`.
21. Check provider route: `oya ops intelligence provider route-history --tenant $TENANT --since 60m --output table`.
22. Check tool policy: `oya ops intelligence tool-policy explain --tenant $TENANT --tool <tool> --output yaml`.
23. Check RAG corpus chunk: `oya ops intelligence rag chunk inspect --tenant $TENANT --chunk <chunk-id> --redacted --output json`.
24. Check eval gate: `oya ops intelligence eval-gate status --tenant $TENANT --pack $PACK --output json`.
25. Run safety replay dry-run: `oya ops intelligence safety replay --tenant $TENANT --incident $INCIDENT_ID --dry-run`.
26. Run canonicalen-set subset: `oya ops intelligence eval run --pack $PACK --suite prompt-fence-critical --incident $INCIDENT_ID --output json`.
27. Query prompt fence audit: `oya audit-chain query --event-class intelligence.prompt_fence.checked --tenant $TENANT --since 24h`.
28. Query refusal audit: `oya audit-chain query --event-class intelligence.refusal.decision --tenant $TENANT --since 24h`.
29. Query tool audit: `oya audit-chain query --event-class intelligence.tool_call.authorized --tenant $TENANT --since 24h`.
30. Check support case handling: `oya support cases list --tag ai-safety --tenant $TENANT --since 24h`.
31. Check customer-visible outputs: `oya ops intelligence output containment status --tenant $TENANT --incident $INCIDENT_ID --output json`.
32. Check active related runbooks: `oya incident list --service intelligence --tag prompt-injection --state active`.
33. Snapshot evidence: `oya evidence snapshot --incident $INCIDENT_ID --microservice intelligence --runbook prompt-fence-bypass-detection --output evidence/incidents/$INCIDENT_ID.json`.
34. Export redacted incident bundle: `oya ops intelligence prompt-fence incident-bundle --tenant $TENANT --incident $INCIDENT_ID --redacted --output evidence/incidents/$INCIDENT_ID-bundle.json`.
35. Export eval results: `oya ops intelligence eval export --incident $INCIDENT_ID --suite prompt-fence-critical --output evidence/incidents/$INCIDENT_ID-eval.json`.

### Diagnostic Decision Tree
```text
1. Did bypass reach a customer-visible output or tool call?
   |-- yes: keep Sev0, contain output, and page security/compliance.
   |-- no: keep Sev1 and confirm block evidence.
2. Is bypass tied to one model route or provider?
   |-- yes: disable that route and rerun eval.
   |-- no: inspect prompt fence bundle and tool policy.
3. Is RAG corpus text the adversarial source?
   |-- yes: quarantine chunk and invoke RAG corpus drift runbook if broad.
   |-- no: inspect prompt template and guardrail classifier.
4. Did eval gate miss the regression?
   |-- yes: block promotion and add adversarial fixture.
   |-- no: patch runtime guardrail or policy.
5. Are prompt fence audit events missing?
   |-- yes: keep incident open and replay audit tap.
   |-- no: close after containment and regression.
```

## Mitigation
1. Block affected model route: `oya ops intelligence model-router disable-route --tenant $TENANT --route <route-id> --ttl 30m --reason $INCIDENT_ID`.
2. Pin prior prompt fence bundle: `oya ops intelligence prompt-fence bundle pin --tenant $TENANT --pack $PACK --version previous-stable --reason $INCIDENT_ID`.
3. Disable unsafe tool: `oya ops intelligence tool-policy disable --tenant $TENANT --tool <tool> --ttl 30m --reason $INCIDENT_ID`.
4. Quarantine RAG chunk: `oya ops intelligence rag chunk quarantine --tenant $TENANT --chunk <chunk-id> --reason $INCIDENT_ID`.
5. Contain output: `oya ops intelligence output contain --tenant $TENANT --incident $INCIDENT_ID --confirm`.
6. Hold prompt/policy deploys: incident hold PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
7. Increase refusal strictness: `oya flags set oya.intelligence.prompt_fence.strict_mode=true --tenant $TENANT --pack $PACK --reason $INCIDENT_ID`.
8. Keep audit tap required: `oya flags set oya.intelligence.audit_tap.required=true --tenant $TENANT --cell $CELL --reason $INCIDENT_ID`.
9. Run safety replay confirmed: `oya ops intelligence safety replay --tenant $TENANT --incident $INCIDENT_ID --confirm`.
10. Add temporary deny: `oya ops intelligence tool-policy deny-fragment add --tenant $TENANT --tool <tool> --reason $INCIDENT_ID`.
11. Notify support: `oya notify support --incident $INCIDENT_ID --template ai-safety-containment`.
12. Notify tenant admin when output was visible: `oya notify tenant-admin --tenant $TENANT --incident $INCIDENT_ID --template ai-safety-output-contained`.
13. Notify compliance: `oya notify compliance --incident $INCIDENT_ID --category ai-safety-bypass`.
14. Notify security: `oya notify security --incident $INCIDENT_ID --category prompt-fence-bypass`.
15. Emit mitigation audit: `oya audit-chain emit --event-class EVT_INTELLIGENCE_PROMPT_FENCE_BYPASS_DETECTION_INCIDENT --incident $INCIDENT_ID --field mitigation=route-contained`.
16. Preserve raw prompts only in evidence storage.
17. Keep raw sensitive content out of chat, PRs, and tickets.
18. Keep route disablement tenant-scoped unless fleet evidence exists.
19. Keep strict mode until eval regression passes.
20. Keep customer copy reviewed by AI safety.

## Resolution
1. Patch prompt fence bundle if boundary text or separators failed.
2. Patch guardrail classifier if bypass detector missed output.
3. Patch tool policy Cedar if unauthorized tool action executed.
4. Patch RAG chunk sanitizer if corpus text became instruction.
5. Patch eval gate if regression was not caught.
6. Patch audit tap if prompt fence checks were not emitted.
7. Add adversarial fixture from redacted incident bundle.
8. Add model-route-specific eval case.
9. Run guardrail tests: `cargo test -p oya-governance-eval-domain prompt_fence -- --nocapture`.
10. Run routing safety tests: `cargo test -p oya-intelligence-route-policy-kernel prompt_fence -- --nocapture`.
11. Run production gate: `cargo run -p oya-dev-cli -- gate validate intelligence-prompt-fence --production-snapshot --cell $CELL`.
12. Verify eval set: `oya ops intelligence eval run --pack $PACK --suite prompt-fence-critical --expect pass`.
13. Remove temporary deny only after permanent policy lands: `oya ops intelligence tool-policy deny-fragment remove --tenant $TENANT --tool <tool> --reason resolved-$INCIDENT_ID`.
14. Unhold deploys: recovery PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
15. Seal audit: `oya audit-chain emit --event-class EVT_INTELLIGENCE_PROMPT_FENCE_BYPASS_DETECTION_INCIDENT --incident $INCIDENT_ID --field resolution=complete`.

## Verification Checklist
- `IntelligencePromptFenceBypassDetectedCritical` is green.
- `oya_intelligence_refusal_false_negative_rate` is within SLO.
- Prompt fence bundle digest matches approved version.
- Affected model route has safe eval result.
- Tool policy denies the bypass path.
- RAG chunk is quarantined or sanitized when implicated.
- Output containment status is complete.
- Audit-chain contains prompt fence checks, mitigation, and resolution.
- Raw sensitive content is stored only in evidence storage.
- AI safety signs off before close.

## Postmortem Template
```markdown
---
doc_class: IncidentPostmortem
runbook_id: intelligence-prompt-fence-bypass-detection
microservice: intelligence
event_class: EVT_INTELLIGENCE_PROMPT_FENCE_BYPASS_DETECTION_INCIDENT
incident_id: <INC-...>
severity: sev0
status: draft
detected_at: <UTC>
mitigated_at: <UTC>
resolved_at: <UTC>
commander: <handle>
evidence_hash: <sha256>
---

# Prompt Fence Bypass Detection postmortem

## Summary
- Which tenant, pack, model route, prompt fence, and tool policy were involved.
- Whether bypass reached customer output or tool execution.
- Whether RAG corpus or prompt template caused it.

## Timeline
- Detection:
- Containment:
- Eval fixture added:
- Policy patched:
- Audit sealed:

## Safety Impact
- Output visible:
- Tool executed:
- Data exposed:
- Customer notice:

## Root Cause
- Prompt fence:
- Classifier:
- Tool policy:
- RAG corpus:
- Eval gate:

## Corrective Actions
- Owner:
- Due date:
- Regression test:
- Policy update:
```

## Escalation Path
- Page `oya-ai-safety-primary` for all prompt fence bypass detections.
- Page `oya-intelligence-primary` for runtime mitigation.
- Page `oya-security-policy-primary` for tool or secret boundary risk.
- Page `oya-audit-chain-primary` when prompt fence events are missing.
- Page `oya-compliance-primary` when regulated output or user data is involved.
- Notify `#inc-ai-safety` with tenant, pack, route, and containment state.
- Notify `#support-ai-safety` before customer communication.
- Notify `#privacy-review` if prompts or outputs include personal data.
- Escalate to executive incident commander if bypass is fleet-wide or public.
- Keep sensitive evidence out of incident chat.

## Cross-µservice Coordination
- `audit-chain`: seal prompt fence, refusal, tool, mitigation, and resolution events.
- `cloud-iam`: verify tool and operator authorization.
- `cloud-kms`: verify BYOK route or credential boundary if implicated.
- `cloud-network`: verify provider egress was not redirected.
- `cloud-billing`: annotate potential SLA or credit when service was disabled.
- `tenancy`: verify pack, region, and regulated tenant classification.
- `support`: manage customer-visible safety cases.
- `ai-safety`: own containment and eval sign-off.
- `security`: own tool, secret, and abuse investigation.
- `compliance`: decide notification obligations.
- `observability`: attach prompt fence and refusal dashboards.
- `foundry`: pause prompt bundle or guardrail deploys.

## Runbook Maintenance
- Add new bypass pattern signatures after every incident.
- Keep sensitive evidence handling explicit.
- Keep eval set names aligned with CI.
- Review this runbook after every prompt bundle change.
- Add every new tool policy surface to Diagnostic Steps.
