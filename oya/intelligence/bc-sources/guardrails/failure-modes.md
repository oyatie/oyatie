---
doc_class: FailureModeCatalog
title: Failure-Mode Catalog
microservice: foundry-guardrails
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-foundry-guardrails
deciders: ops-sre-reliability, axis-foundry-guardrails, ops-security, council-architecture
related_adrs: [ADR-0022, ADR-0117, ADR-0139, ADR-0131, ADR-0140 (retired per ADR-0145)]
related_artifacts:
  - microservices/intelligence-guardrails/threat-model.md
  - microservices/intelligence-guardrails/dpia.md
  - microservices/intelligence-guardrails/policy/tenant-isolation.md
  - microservices/intelligence-guardrails/incident-response.md
  - microservices/intelligence-guardrails/runbooks/
review_cadence: quarterly + after every Sev-1 / Sev-2 incident affecting foundry-guardrails
doc_status: published
---

# Failure-Mode Catalog (foundry-guardrails µservice)

## Purpose

Enumerate failure scenarios on-call must handle: trigger; detection SLI; tenant impact; severity; immediate mitigation; RTO; recovery runbook; postmortem owner.

## Index

Each carries: **FM-ID**, **Trigger**, **Detection**, **Tenant impact**, **Severity** (per `incident-response.md`), **Immediate mitigation**, **RTO**, **Recovery runbook**, **Postmortem owner**.

## FM-01: Classifier-model-serving pod outage (single model, single AZ)

| Field | Value |
|---|---|
| Trigger | Cluster eviction, hardware failure, kernel panic, OOM kill of all classifier pods for one model in one AZ |
| Detection | `foundry_guardrails_classifier_request_duration_seconds{model="<m>", quantile="0.99"} > 80ms` for ≥ 3 min OR replica-count drops below HA minimum |
| Tenant impact | Latency spike on pre-invocation classify; ensemble falls back to heuristic-only mode with degraded-confidence flag |
| Severity | Sev-2 |
| Immediate mitigation | HPA scale up surviving replicas; cordon affected AZ; HA min suffices for short outage |
| RTO | ≤ 5 min |
| Recovery runbook | `runbooks/classifier-model-rollback.md` |
| Postmortem owner | axis-foundry-guardrails |

## FM-02: Classifier-model rollout regression (new model worse than prior on shadow metrics)

| Field | Value |
|---|---|
| Trigger | Promote-to-enforce attempt of a model whose shadow-vs-enforce delta on baseline fixtures exceeds tolerance |
| Detection | `foundry_guardrails_shadow_vs_enforce_delta{model="<m>"} > 0.05` (5% absolute decision change) |
| Tenant impact | Caught pre-promote; zero tenant impact |
| Severity | Sev-2 (operational gate; safe default applies) |
| Immediate mitigation | Refuse promote; rule-author reviews shadow metrics; iterate |
| RTO | n/a (caught pre-promote) |
| Recovery runbook | `runbooks/classifier-model-rollback.md` §"Shadow regression" |
| Postmortem owner | axis-foundry-guardrails |

## FM-03: Classifier-serving outage (full pool exhausted)

| Field | Value |
|---|---|
| Trigger | Coordinated multi-AZ failure; model integrity-check fails on every replica; provider-level OCI outage |
| Detection | `foundry_guardrails_classifier_pool_healthy_pct < 50%` for ≥ 2 min |
| Tenant impact | Pre-invocation classify fail-closed (block every invocation); tenant agents stop responding |
| Severity | Sev-1 |
| Immediate mitigation | Engage ops-sre + axis-foundry-guardrails; if model integrity issue: roll back to prior model SHA via runbook; if provider outage: DR failover (DR-pair packs); emergency-bypass entitlement for high-trust tenants (heuristic-only) |
| RTO | ≤ 15 min for DR failover; ≤ 5 min for model rollback |
| Recovery runbook | `runbooks/classifier-model-rollback.md` + `runbooks/dr-failover-drill.md` |
| Postmortem owner | ops-sre-reliability + axis-foundry-guardrails |

## FM-04: Cedar engine evaluation timeout

| Field | Value |
|---|---|
| Trigger | Cedar bundle bug (infinite loop in fragment); oversized policy bundle; oversized input |
| Detection | `foundry_guardrails_cedar_evaluation_timeout_total > 0` over 5 min |
| Tenant impact | Per-evaluation timeout → fail-closed (block); affected invocations refused |
| Severity | Sev-2 if isolated; Sev-1 if cluster-wide |
| Immediate mitigation | Roll back Cedar bundle to prior SHA via `runbooks/cedar-engine-restart.md`; isolate offending fragment |
| RTO | ≤ 10 min for bundle rollback |
| Recovery runbook | `runbooks/cedar-engine-restart.md` |
| Postmortem owner | axis-foundry-guardrails |

## FM-05: Cedar default-deny config drift (someone weakens base deny rule)

| Field | Value |
|---|---|
| Trigger | Helm config change merged without lane gate; OR live-cluster mutation |
| Detection | `oya-governance-cedar-default-deny-enforced` lane fails OR continuous Helm-state-validator alarms |
| Tenant impact | Potential cross-tenant entitlement leakage if not caught pre-deploy |
| Severity | Sev-1 (security risk) |
| Immediate mitigation | Auto-rollback to last green Cedar bundle via ArgoCD; isolate cluster; engage ops-security |
| RTO | ≤ 5 min auto-rollback; investigation may take days |
| Recovery runbook | `runbooks/cedar-engine-restart.md` §"Default-deny drift" + ops-security incident |
| Postmortem owner | ops-security + axis-foundry-guardrails |

## FM-06: Sev-1 jailbreak success (false-negative; unsafe content reached caller)

| Field | Value |
|---|---|
| Trigger | A prompt classified as safe by all ensemble members induced a provider output containing unsafe content; detected post-hoc via output-validator OR tenant report OR red-team |
| Detection | `foundry_guardrails_sev1_jailbreak_total > 0` OR FP escalation queue receives entry tagged `false_negative_severe` |
| Tenant impact | Tenant + tenant-of-tenant may have been exposed to unsafe content |
| Severity | Sev-1 (always for confirmed jailbreak success) |
| Immediate mitigation | Engage axis-foundry-guardrails IC; freeze the offending capability for the affected tenant; auto-allocate incident ID; auto-generate post-mortem template; pin failing prompt to red-team fixtures |
| RTO | ≤ 5 min freeze; investigation + retrain may take days |
| Recovery runbook | `runbooks/jailbreak-escalation.md` |
| Postmortem owner | axis-foundry-guardrails |

## FM-07: False-positive surge (rule overly aggressive after rollout)

| Field | Value |
|---|---|
| Trigger | New rule promoted to enforce; FP rate spikes |
| Detection | `foundry_guardrails_false_positive_rate{rule_id="<r>"} > 0.05` for ≥ 30 min |
| Tenant impact | Legitimate prompts blocked; tenant operators see surge in escalation budget consumption |
| Severity | Sev-2 |
| Immediate mitigation | Roll back rule to prior version per `runbooks/policy-rule-rollback.md`; tenant operators auto-notified |
| RTO | ≤ 10 min rule rollback |
| Recovery runbook | `runbooks/policy-rule-rollback.md` |
| Postmortem owner | axis-foundry-guardrails |

## FM-08: False-positive escalation budget exhausted (tenant)

| Field | Value |
|---|---|
| Trigger | A single tenant marks > N blocks/month as FP (tier-specific budget) |
| Detection | `foundry_guardrails_tenant_fp_budget_consumed_pct > 100` |
| Tenant impact | Tenant continues to see blocks; FP-marking returns budget-exceeded error; rule-author queue surfaces |
| Severity | Sev-3 (operational; one tenant) |
| Immediate mitigation | Engage axis-foundry-guardrails on-call; review tenant's FP entries; promote selective rule adjustments via shadow→enforce |
| RTO | per-tenant SLA (24-72h typically) |
| Recovery runbook | `runbooks/false-positive-tenant-relief.md` |
| Postmortem owner | axis-foundry-guardrails |

## FM-09: Postgres rule-store unavailable

| Field | Value |
|---|---|
| Trigger | Primary Postgres crash; AZ outage; storage failure |
| Detection | `foundry_guardrails_rule_store_request_failures_total > 0` AND `pg_replication_lag_seconds > 5` |
| Tenant impact | Rule fetch falls back to in-pod cache (5s TTL); if cache stale, rule evaluation may use stale rules briefly; fail-closed when cache expired AND PG unreachable |
| Severity | Sev-2 (DR-pair packs auto-promote read replica); Sev-1 (single-region packs without DR) |
| Immediate mitigation | DR-pair: promote read replica per `runbooks/dr-failover-drill.md`; single-region: provider-dependent recovery |
| RTO | ≤ 3 min RR promotion (DR-pair); ≤ 1h single-region |
| Recovery runbook | `runbooks/rule-store-restore.md` |
| Postmortem owner | ops-sre-reliability + axis-foundry-guardrails |

## FM-10: Rule-store backup corruption

| Field | Value |
|---|---|
| Trigger | Backup-validation lane detects corruption on a Postgres dump |
| Detection | `foundry_guardrails_rule_store_backup_validation_failures_total > 0` (daily lane) |
| Tenant impact | None yet; backup is a defence-in-depth |
| Severity | Sev-2 |
| Immediate mitigation | Quarantine corrupted backup; restore from prior validated backup; investigate corruption cause |
| RTO | ≤ 1h backup restore |
| Recovery runbook | `runbooks/rule-store-restore.md` |
| Postmortem owner | ops-sre-reliability + axis-foundry-guardrails |

## FM-11: LLM-judge fallback budget exhaustion (per-tenant)

| Field | Value |
|---|---|
| Trigger | Single tenant's LLM-judge invocations exceed hourly budget |
| Detection | `foundry_guardrails_llm_judge_budget_exceeded_total > 0` |
| Tenant impact | Ambiguous prompts fail-closed (block + budget-exceeded reason); tenant agent partially degraded |
| Severity | Sev-3 (tenant-bounded) |
| Immediate mitigation | Per-tenant FP escalation review; rule-author tunes ensemble disagreement threshold to reduce LLM-judge invocation rate |
| RTO | next billing hour |
| Recovery runbook | `runbooks/false-positive-tenant-relief.md` §"LLM-judge budget" |
| Postmortem owner | axis-foundry-guardrails + axis-foundry-providers |

## FM-12: Cross-tenant rule leak detected

| Field | Value |
|---|---|
| Trigger | LEAN check or runtime audit detects tenant-A's rule overlay applied to tenant-B request |
| Detection | `oya_tenant_unauthorized_rule_apply_total > 0` OR continuous-compliance lane alarm |
| Tenant impact | Potential confidentiality breach (DPIA R-05; threat T-I-03) |
| Severity | Sev-1 (security) |
| Immediate mitigation | Engage ops-security; freeze affected REST endpoint; revoke implicated tokens; begin forensic trace |
| RTO | ≤ 5 min freeze; investigation + breach-notification chain per GDPR Art. 33 may take 72h+ |
| Recovery runbook | ops-security incident playbook + `incident-response.md` §"Severity 1" |
| Postmortem owner | ops-security |

## FM-13: Pack misroute (tenant's prompt sent to wrong-pack guardrails)

| Field | Value |
|---|---|
| Trigger | foundry-runtime pack-routing bug; OTel collector mis-tag |
| Detection | `foundry_guardrails_pack_mismatch_total > 0` |
| Tenant impact | Potential residency breach (DPIA R-11); GDPR Art. 44 violation if EU→non-EU |
| Severity | Sev-1 (regulatory) |
| Immediate mitigation | Freeze offending route; engage council-privacy + ops-security; begin breach-notification chain |
| RTO | ≤ 5 min freeze; investigation + notification 72h |
| Recovery runbook | `incident-response.md` §"Severity 1 — Regulatory" + foundry-runtime pack-routing runbook |
| Postmortem owner | ops-security + council-privacy + axis-foundry-runtime |

## FM-14: Classifier-model integrity violation (Cosign signature mismatch at pod start)

| Field | Value |
|---|---|
| Trigger | Tampered model artifact OR Cosign signing-key rotation issue OR mis-published artifact |
| Detection | Pod fails to start; `foundry_guardrails_classifier_model_integrity_violation_total > 0` |
| Tenant impact | Affected model pods stay down; HA replicas absorb load if other AZs healthy |
| Severity | Sev-1 if cluster-wide; Sev-2 if isolated |
| Immediate mitigation | Engage ops-security; verify Cosign key state in OpenBao; roll back to prior signed model SHA; investigate artifact origin |
| RTO | ≤ 5 min rollback |
| Recovery runbook | `runbooks/classifier-model-rollback.md` §"Integrity violation" |
| Postmortem owner | ops-security + axis-foundry-guardrails |

## FM-15: foundry-runtime stops calling guardrails (coupling lane fails post-deploy)

| Field | Value |
|---|---|
| Trigger | Coding bug in foundry-runtime; or intentional bypass; CI lane caught pre-deploy normally |
| Detection | `oya-governance-runtime-guardrails-coupling` lane fails OR runtime audit shows foundry-runtime → foundry-providers calls without guardrails round-trip |
| Tenant impact | Catastrophic: every invocation bypasses safety floor |
| Severity | Sev-1 (always) |
| Immediate mitigation | Roll back foundry-runtime to last-good deploy; engage axis-foundry + axis-foundry-guardrails ICs jointly |
| RTO | ≤ 15 min rollback (foundry-runtime auto-rollback per ADR-0139) |
| Recovery runbook | foundry-runtime's `runbooks/runtime-rollback.md` + this µservice's `runbooks/jailbreak-escalation.md` (because every bypassed invocation is potentially a missed jailbreak) |
| Postmortem owner | axis-foundry-runtime + axis-foundry-guardrails (joint) |

## Cross-Reference Summary

| Severity | Failure modes |
|---|---|
| Sev-1 | FM-03, FM-05, FM-06, FM-12, FM-13, FM-14 (cluster-wide), FM-15 |
| Sev-2 | FM-01, FM-02, FM-04, FM-07, FM-09 (DR-pair), FM-10, FM-14 (isolated) |
| Sev-3 | FM-08, FM-11 |
| Sev-4 | (none yet) |

## References

- `microservices/intelligence-guardrails/threat-model.md`.
- `microservices/intelligence-guardrails/dpia.md`.
- `microservices/intelligence-guardrails/incident-response.md`.
- `microservices/intelligence-guardrails/runbooks/`.
- `microservices/observability/failure-modes.md` (sibling reference shape).
