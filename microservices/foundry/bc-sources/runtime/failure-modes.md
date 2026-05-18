---
doc_class: FailureModeCatalog
title: Failure-Mode Catalog
microservice: foundry-runtime
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-foundry-runtime
deciders: ops-sre-reliability, axis-foundry-runtime, ops-security, council-architecture
related_adrs: [ADR-0022, ADR-0025, ADR-0117, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/foundry-runtime/threat-model.md
  - microservices/foundry-runtime/dpia.md
  - microservices/foundry-runtime/policy/runtime-isolation.md
  - microservices/foundry-runtime/incident-response.md
  - microservices/foundry-runtime/runbooks/
review_cadence: quarterly + after every Sev-1 / Sev-2 incident affecting foundry-runtime
doc_status: published
---

# Failure-Mode Catalog (foundry-runtime µservice)

## Purpose

Enumerate failure scenarios on-call must handle. Each carries: trigger; detection signal; tenant impact; severity; immediate mitigation; RTO; recovery runbook; postmortem owner. Cross-referenced from `incident-response.md` for severity classification.

## Failure-Mode Index

## FM-01: Runtime pod crashloop (single pod)

| Field | Value |
|---|---|
| Trigger | OOM kill (LLM response too large), panic on malformed capability descriptor, OpenBao token-renewal failure |
| Detection | `kube_pod_status_phase{phase="Running",pod=~"oya-foundry-runtime-.*"}` drops; `oya_foundry_runtime_pod_restarts_total` rises |
| Tenant impact | In-flight invocations on the failing pod re-dispatched to peer pods within ≤5s |
| Severity | Sev-3 single pod; Sev-2 if pattern (3+ pods in 5min) |
| Immediate mitigation | Verify HPA scaling; cordon affected node if pattern; restart pod |
| RTO | ≤5min for single pod; ≤15min for pattern |
| Recovery runbook | `runbooks/runtime-pod-crash.md` |
| Postmortem owner | axis-foundry-runtime |

## FM-02: Session-state Redis cluster partition (single AZ)

| Field | Value |
|---|---|
| Trigger | AZ network partition; OCI hardware failure |
| Detection | `oya_foundry_runtime_redis_connection_failures_total > threshold` for ≥3min; some Redis shards report majority loss |
| Tenant impact | Hot session reads latency-degraded on affected shards; sessions still recoverable from Postgres cold restore |
| Severity | Sev-2 |
| Immediate mitigation | Verify replication promote; HPA scales runtime-pool to absorb cold-restore latency hit; cordon affected AZ |
| RTO | ≤15min for shard recovery; ≤30min for full AZ restoration |
| Recovery runbook | `runbooks/redis-failover.md` |
| Postmortem owner | ops-sre-reliability + axis-foundry-runtime |

## FM-03: Session-state Redis ACL drift

| Field | Value |
|---|---|
| Trigger | Helm config change merged without lane gate; OR live-cluster mutation by admin |
| Detection | `oya-check-session-prefix-isolation` lane fails OR continuous Helm-state-validator alarms |
| Tenant impact | Potential cross-tenant session exposure if not caught pre-deploy |
| Severity | Sev-1 (security breach risk) |
| Immediate mitigation | Auto-rollback to last green Helm state via ArgoCD; isolate Redis; engage ops-security; declare Sev-1 |
| RTO | ≤5min auto-rollback; investigation may take days |
| Recovery runbook | `runbooks/redis-failover.md` §"ACL drift" + ops-security incident playbook |
| Postmortem owner | ops-security + axis-foundry-runtime |

## FM-04: Capability registry cache stale (foundry-supervisor unreachable)

| Field | Value |
|---|---|
| Trigger | foundry-supervisor outage; replication path broken |
| Detection | `oya_foundry_runtime_registry_cache_age_seconds > 60` (cache age > heartbeat window) |
| Tenant impact | Newly-registered capability descriptors not yet visible; existing capabilities continue serving with last-known version |
| Severity | Sev-2 (gate fails-graceful: continue with cached descriptor; tenant-newer-descriptor latency uptick) |
| Immediate mitigation | Verify supervisor reachability; failover to supervisor DR replica if exists; tenant comms if registration delay > 30min |
| RTO | ≤30min for supervisor restoration; cache stays usable indefinitely with degraded freshness |
| Recovery runbook | `runbooks/capability-registry-resync.md` |
| Postmortem owner | axis-foundry-runtime + axis-foundry |

## FM-05: Capability descriptor signature invalid (T-T-04 in threat-model)

| Field | Value |
|---|---|
| Trigger | Replication corruption, tampering attempt, supervisor signing-key rotation gone wrong |
| Detection | `oya_capability_mirror_signature_invalid_total > 0` |
| Tenant impact | Affected capability dispatch refused (fail-closed); tenant comms required |
| Severity | Sev-1 (potential tampering) — Sev-2 if root cause is rotation timing |
| Immediate mitigation | Blacklist affected descriptor; engage ops-security; investigate; pull-and-verify from supervisor |
| RTO | ≤30min for legitimate cases; security incident if tampering confirmed |
| Recovery runbook | `runbooks/capability-registry-resync.md` §"Signature invalid" |
| Postmortem owner | ops-security + axis-foundry-runtime |

## FM-06: Autonomy violation surge (T-E-01 in threat-model)

| Field | Value |
|---|---|
| Trigger | Misbehaving tenant/workload repeatedly dispatching above-ceiling capabilities; OR malicious probing |
| Detection | `rate(oya_foundry_runtime_autonomy_violation_total[5m]) > threshold` per tenant |
| Tenant impact | Dispatch refused per violation; tenant-facing 403; tenant operator informed |
| Severity | Sev-2 (security signal) |
| Immediate mitigation | Verify violations are not false-positive (ceiling cache stale?); engage tenant operator; if malicious, ops-security incident |
| RTO | ≤30min for tenant comms; ≤24h for tenant remediation |
| Recovery runbook | `runbooks/autonomy-violation-quarantine.md` |
| Postmortem owner | ops-security + axis-foundry-runtime |

## FM-07: Provider credential leakage probe positive (T-I-03 in threat-model)

| Field | Value |
|---|---|
| Trigger | Secret-scanner CI lane flags a runtime log emission OR coredump containing provider secret pattern |
| Detection | `oya_foundry_runtime_provider_secret_leak_attempted_total > 0` OR CI lane fails |
| Tenant impact | DPIA R-17 risk realised; provider integration may need credential rotation |
| Severity | Sev-1 (data-protection breach risk) |
| Immediate mitigation | Engage ops-security; rotate provider credentials at foundry-providers (drains in-flight invocations bound to old gen); purge affected logs (Loki delete); harden log redactor |
| RTO | ≤1h rotation + redactor patch; ≤24h log purge |
| Recovery runbook | `runbooks/security-incident.md` §"Provider credential leak" |
| Postmortem owner | ops-security + foundry-providers + axis-foundry-runtime |

## FM-08: Sibling µservice unreachable (foundry-providers / -guardrails / -evidence)

| Field | Value |
|---|---|
| Trigger | Sibling outage; mTLS handshake failure; Istio mesh issue |
| Detection | Circuit-breaker per sibling opens; `oya_foundry_runtime_sibling_failures_total{sibling="..."} > threshold` |
| Tenant impact | Per affected sibling: providers down → invocation fails with `InvocationFailed{reason=provider_unreachable}`; guardrails down → fail-closed (refuse dispatch); evidence down → continue but degrade evidence emission (queue + retry) |
| Severity | Sev-2 |
| Immediate mitigation | Engage sibling on-call; circuit-breaker absorbs; if guardrails down + critical workflow, surface to ops-sre for manual override path |
| RTO | depends on sibling; runtime side ≤5min circuit-breaker; sibling recovery per its own RTO |
| Recovery runbook | `runbooks/runtime-pod-crash.md` §"Sibling unreachable" |
| Postmortem owner | the affected sibling owner |

## FM-09: Postgres mirror replica fail (capability mirror or invocation lifecycle)

| Field | Value |
|---|---|
| Trigger | Postgres replica crash; replication lag > threshold |
| Detection | `pg_replication_lag_seconds > 60` OR replica unreachable |
| Tenant impact | Primary continues; lifecycle record reads serve primary; capability mirror reads serve primary |
| Severity | Sev-3 (degraded read distribution) |
| Immediate mitigation | Restart replica; verify replication; if pattern, scale up replica count |
| RTO | ≤30min |
| Recovery runbook | `runbooks/session-state-recovery.md` §"Postgres replica fail" |
| Postmortem owner | ops-sre-reliability |

## FM-10: Session-state cold restore latency spike (Postgres slow)

| Field | Value |
|---|---|
| Trigger | Postgres-side latency uptick (vacuum, lock contention, IO saturation) |
| Detection | `oya_foundry_runtime_session_cold_restore_duration_seconds{quantile="0.99"} > 100ms` |
| Tenant impact | Session resume slower on Redis miss; tenant-facing latency observable |
| Severity | Sev-3 |
| Immediate mitigation | Investigate Postgres slow log; tune autovacuum; scale Postgres vertically if pattern |
| RTO | ≤1h for tuning; vertical scale ≤30min |
| Recovery runbook | `runbooks/session-state-recovery.md` §"Cold restore latency" |
| Postmortem owner | ops-sre-reliability + axis-foundry-runtime |

## FM-11: Pod drain failure (in-flight invocations not parked safely)

| Field | Value |
|---|---|
| Trigger | Pod terminated before drain completes (e.g., SIGKILL after grace period) |
| Detection | `oya_foundry_runtime_invocations_lost_in_drain_total > 0` |
| Tenant impact | Affected invocations emit `InvocationFailed{reason=runtime_drain_lost}`; tenant retries |
| Severity | Sev-2 if pattern; Sev-3 single incident |
| Immediate mitigation | Increase pod grace period; verify drain procedure honours longest in-flight budget |
| RTO | ≤15min for grace-period reconfig |
| Recovery runbook | `runbooks/emergency-runtime-drain.md` §"Drain failure" |
| Postmortem owner | axis-foundry-runtime + ops-sre-reliability |

## FM-12: Cross-tenant session leak detected (T-I-01)

| Field | Value |
|---|---|
| Trigger | LEAN check or runtime audit detects cross-tenant session read; Redis prefix bypass |
| Detection | `oya_foundry_runtime_unauthorized_attempt_total > 0` over 5min OR continuous-compliance lane alarm |
| Tenant impact | Potential confidentiality breach (DPIA R-02; threat T-I-01) |
| Severity | Sev-1 (security breach) |
| Immediate mitigation | Engage ops-security; freeze affected REST endpoint; revoke implicated API keys; forensic trace |
| RTO | ≤5min for endpoint freeze; investigation + breach-notification may take 72h+ |
| Recovery runbook | `runbooks/security-incident.md` (cross-references `incident-response.md` §"Severity 1") |
| Postmortem owner | ops-security |

## FM-13: Prompt-injection causes session contamination (T-I-02)

| Field | Value |
|---|---|
| Trigger | Adversarial input bypasses guardrails; session conversation history influenced |
| Detection | `oya_foundry_runtime_guardrail_miss_total > 0`; tenant report |
| Tenant impact | Affected session(s) may carry adversarial context; capability outputs unreliable |
| Severity | Sev-2 (data-integrity); Sev-1 if cross-tenant contamination |
| Immediate mitigation | Quarantine affected sessions; tighten guardrail ruleset; tenant comms + offer session reset |
| RTO | ≤1h guardrail update; ≤24h tenant session reset |
| Recovery runbook | `runbooks/security-incident.md` §"Prompt-injection contamination" |
| Postmortem owner | ops-security + foundry-guardrails + axis-foundry-runtime |

## FM-14: Capacity exhaustion (pool saturated; 429 surge)

| Field | Value |
|---|---|
| Trigger | Burst load exceeds per-tenant or per-pack rate limits |
| Detection | `oya_foundry_runtime_rate_limit_exceeded_total` rate climbs; HPA at ceiling |
| Tenant impact | New invocations return 429 + Retry-After; existing invocations complete normally |
| Severity | Sev-3 (tenant-specific) — Sev-2 if cluster-wide |
| Immediate mitigation | Verify HPA at ceiling; increase ceiling if production-tier and within global budget; engage tenant on rate-limit tuning |
| RTO | ≤30min for ceiling increase; tenant remediation may take days |
| Recovery runbook | `runbooks/runtime-pod-crash.md` §"Capacity exhaustion" |
| Postmortem owner | ops-sre-reliability |

## FM-15: Long-running invocation starves pod (T-D-03)

| Field | Value |
|---|---|
| Trigger | Capability with mis-set timeout; runaway tool-call loop |
| Detection | `oya_foundry_runtime_invocation_duration_seconds{quantile="0.99"} > capability_timeout_max` |
| Tenant impact | Pod concurrency reduced; eventual `InvocationFailed{reason=timeout}` |
| Severity | Sev-3 (degraded; safety net working) |
| Immediate mitigation | TimeoutClock enforces; investigate offending capability; tighten descriptor timeout |
| RTO | ≤15min for timeout enforcement |
| Recovery runbook | `runbooks/runtime-pod-crash.md` §"Long invocation" |
| Postmortem owner | axis-foundry-runtime |

## RTO / RPO Summary

| Failure | RTO | RPO |
|---|---|---|
| FM-01 Pod crashloop | 5–15min | 0 (peer rebalance) |
| FM-02 Redis cluster partition | 15min shard / 30min AZ | ≤30s (last AOF flush) |
| FM-03 Redis ACL drift | 5min auto-rollback | 0 |
| FM-04 Registry cache stale | 30min | 0 (cache stays usable) |
| FM-05 Descriptor signature invalid | 30min | 0 |
| FM-06 Autonomy violation surge | 30min tenant comms | N/A |
| FM-07 Provider credential leak | 1h rotation + 24h purge | N/A |
| FM-08 Sibling unreachable | per-sibling RTO | N/A |
| FM-09 Postgres replica fail | 30min | 0 (primary serves) |
| FM-10 Cold restore latency | 1h tuning | N/A |
| FM-11 Pod drain failure | 15min reconfig | N/A (invocations explicitly failed) |
| FM-12 Cross-tenant leak | 5min freeze | N/A (breach occurred) |
| FM-13 Prompt-injection contamination | 1h + 24h session reset | N/A |
| FM-14 Capacity exhaustion | 30min ceiling | varies |
| FM-15 Long invocation | 15min timeout | N/A |

## SLO on Failure-Detection Pipeline

Meta-SLO: no failure remains undetected longer than its detection-window target.

| SLI | Target | Burn-rate alert |
|---|---|---|
| Alert-to-page latency (p99) | ≤60s | 14.4× burn over 1h |
| Detection-coverage (synthetic faults caught) | ≥99.5% | 6× burn over 6h |
| Two-channel corroboration completion | ≥99% within 90s | ticket burn 3d |
| False-positive page rate | ≤1 / week / on-call | informational |

## References

- `microservices/foundry-runtime/threat-model.md` (each FM has at least one corresponding STRIDE / LINDDUN threat ID).
- `microservices/foundry-runtime/dpia.md` (FM-12, FM-13, FM-07 map to R-02, R-01, R-17).
- `microservices/foundry-runtime/incident-response.md` §"Severity Definitions".
- `microservices/foundry-runtime/runbooks/*`.
- `microservices/foundry-runtime/capacity-model.md`.
- Google SRE Workbook ch. 12 (Postmortem culture).
- OWASP Top 10 for LLM Applications 2025.
