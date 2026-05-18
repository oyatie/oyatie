---
doc_class: FailureModeCatalog
title: Failure-Mode Catalog
microservice: foundry-providers
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-foundry
deciders: ops-sre-reliability, axis-foundry, ops-security, council-architecture
related_adrs: [ADR-0025, ADR-0026, ADR-0117, ADR-0131]
related_artifacts:
  - microservices/foundry-providers/threat-model.md
  - microservices/foundry-providers/dpia.md
  - microservices/foundry-providers/incident-response.md
  - microservices/foundry-providers/runbooks/
review_cadence: quarterly + after every Sev-1 / Sev-2 incident
doc_status: published
---

# Failure-Mode Catalog (foundry-providers µservice)

## Purpose

Enumerate the failure scenarios on-call must handle. Each entry: trigger, detection signal, immediate mitigation, RTO, owning runbook, postmortem owner.

## FM-FP-01: Provider edge outage (single vendor)

| Field | Value |
|---|---|
| Trigger | Vendor (Anthropic / OpenAI / Google) edge degraded or unavailable |
| Detection | `oya_foundry_providers_provider_availability{vendor="<v>"} < 0.95` over 60 s OR `oya_foundry_providers_provider_p99_latency_ms{vendor="<v>"} > 5×baseline` |
| Tenant impact | Brief latency spike; provider-router auto-fails-over to alternate vendor; minimal user-visible impact if alternate available in pack |
| Severity | Sev-2 (single-vendor) / Sev-1 (cascading) |
| Immediate mitigation | provider-router auto-demotes vendor; routes to next-best |
| RTO | ≤ 60 s for router demote; vendor recovery is upstream-bounded |
| Recovery runbook | `runbooks/provider-outage-failover.md` |
| Postmortem owner | axis-foundry |

## FM-FP-02: Rate-limit cascade

| Field | Value |
|---|---|
| Trigger | Tenant exceeds per-vendor rate limit OR vendor throttles shared pool |
| Detection | `oya_foundry_providers_provider_429_total[5m]` exceeds threshold (1 % of requests over 5 min) |
| Tenant impact | Tenant requests delayed or routed to alternate; queue depth grows |
| Severity | Sev-3 (single tenant brief) / Sev-2 (sustained) / Sev-1 (multi-tenant cascade) |
| Immediate mitigation | In-process token bucket refuses; router shifts to alternate vendor |
| RTO | ≤ 60 s for bucket cycle |
| Recovery runbook | `runbooks/rate-limit-cascade-recovery.md` |
| Postmortem owner | axis-foundry + ops-finops |

## FM-FP-03: Credential leak (suspected or confirmed)

| Field | Value |
|---|---|
| Trigger | Credential observed outside OpenBao isolation; anomalous resolution pattern; vendor breach notification |
| Detection | `oya-check-no-raw-credentials` lane fail; OpenBao audit anomaly; OOB report |
| Tenant impact | Potential cross-tenant or external misuse of credential |
| Severity | Sev-1 (confirmed) / Sev-2 (suspected) |
| Immediate mitigation | Emergency revoke per `runbooks/provider-credentials-revoke.md`; rotation per `runbooks/credential-rotation.md` |
| RTO | ≤ 5 min revoke; ≤ 15 min replacement |
| Recovery runbook | `runbooks/provider-credentials-revoke.md` |
| Postmortem owner | ops-security |

## FM-FP-04: In-house model rollout regression

| Field | Value |
|---|---|
| Trigger | In-house model quality drops below incumbent parity floor (default 0.95) |
| Detection | `oya_foundry_providers_provider_quality_score{vendor="in-house"} < 0.95×incumbent` over 60 s |
| Tenant impact | Workload using in-house produces degraded output |
| Severity | Sev-2 (canary cohort) / Sev-1 (production cohort) |
| Immediate mitigation | Router demotes in-house; rolls back to alternate vendor |
| RTO | ≤ 60 s for demote; ≤ 5 min for full rollback |
| Recovery runbook | `runbooks/in-house-model-rollback.md` |
| Postmortem owner | axis-foundry |

## FM-FP-05: Adapter substitution attack (supply chain)

| Field | Value |
|---|---|
| Trigger | Adapter crate replaced with malicious variant in dependency graph |
| Detection | `oya-foundry-providers-adapter-digest-verified` lane fails at deploy; Sigstore attestation mismatch |
| Tenant impact | Potential credential exfil OR response tampering |
| Severity | Sev-1 (security breach) |
| Immediate mitigation | Auto-rollback to previous adapter digest via ArgoCD; isolate cluster |
| RTO | ≤ 5 min auto-rollback |
| Recovery runbook | `runbooks/adapter-version-pin.md` + ops-security incident playbook |
| Postmortem owner | ops-security |

## FM-FP-06: Subscription channel breakage

| Field | Value |
|---|---|
| Trigger | Vendor changes subscription UI / cookie / endpoint without notice |
| Detection | `oya_foundry_providers_response_shape_anomaly_total{vendor="<v>",transport="subscription"}` rises |
| Tenant impact | Subscription-transport tenants degraded; subscription session unavailable until adapter update |
| Severity | Sev-2 (single vendor) |
| Immediate mitigation | Adapter quarantine; router routes tenant to API transport if available; tenant comms |
| RTO | ≤ 60 s router fail-over; adapter fix may take hours |
| Recovery runbook | `runbooks/adapter-version-pin.md` |
| Postmortem owner | axis-foundry |

## FM-FP-07: OpenBao agent unavailable

| Field | Value |
|---|---|
| Trigger | OpenBao agent socket unreachable from adapter pod (e.g., agent pod crashed; mesh degraded) |
| Detection | `oya_foundry_providers_credential_resolve_errors_total[1m]` exceeds threshold |
| Tenant impact | Adapter cannot resolve fresh credentials; lease cache extends grace period until expiry |
| Severity | Sev-1 if lease expires; Sev-2 during grace period |
| Immediate mitigation | OpenBao agent HA — sidecar restarts; fall back to alternate replica |
| RTO | ≤ 5 min HA recovery |
| Recovery runbook | inherited from `cloud-secrets/runbooks/openbao-agent-outage.md` |
| Postmortem owner | ops-security |

## FM-FP-08: Cross-pack mis-route (T-08 realization)

| Field | Value |
|---|---|
| Trigger | Router selects a vendor edge in a non-permitted region (mis-configured residency matrix or Cedar drift) |
| Detection | `oya_foundry_providers_residency_violation_total` ≥ 1 ever; lane `residency-conformance` would also catch pre-deploy |
| Tenant impact | Tenant data may have crossed jurisdictional boundary; regulatory notification potentially required |
| Severity | Sev-1 (security + privacy breach) |
| Immediate mitigation | Block the (pack × vendor × region) tuple; investigate residency-matrix drift; reload Cedar |
| RTO | ≤ 5 min block; investigation timeline by case |
| Recovery runbook | `runbooks/provider-outage-failover.md` + privacy-incident playbook |
| Postmortem owner | council-privacy + axis-foundry |

## FM-FP-09: Provider-router pod outage (single AZ)

| Field | Value |
|---|---|
| Trigger | Cluster eviction / OOM kill of router-rest pods in one AZ |
| Detection | `kube_pod_status_ready{deployment="oya-foundry-providers-router-rest"}` < replica count for ≥ 3 min |
| Tenant impact | Request queue depth grows; latency spike on surviving pods |
| Severity | Sev-2 (degraded; no data loss) |
| Immediate mitigation | HPA scaling triggered; surviving pods absorb load; cordoned AZ |
| RTO | ≤ 15 min |
| Recovery runbook | `runbooks/provider-outage-failover.md` §"Router-pod outage" (cross-link inherits k8s patterns) |
| Postmortem owner | ops-sre-reliability |

## FM-FP-10: Postgres provider-config outage

| Field | Value |
|---|---|
| Trigger | Postgres primary failover or full unavailability |
| Detection | `oya_foundry_providers_postgres_connection_errors_total` rate |
| Tenant impact | Config reads served from in-process cache (stale-while-revalidate); writes blocked |
| Severity | Sev-2 (degraded) |
| Immediate mitigation | Postgres HA failover (replica promoted); in-process cache survives during failover |
| RTO | ≤ 60 s HA failover |
| Recovery runbook | inherited from `cell/runbooks/postgres-failover.md` |
| Postmortem owner | ops-sre-reliability |

## FM-FP-11: Valkey token-bucket outage

| Field | Value |
|---|---|
| Trigger | Valkey primary failover or full unavailability |
| Detection | `oya_foundry_providers_redis_connection_errors_total` rate |
| Tenant impact | Rate-limit enforcement degrades to per-pod local counter (looser limit) during failover |
| Severity | Sev-3 (degraded; not failing) |
| Immediate mitigation | Valkey Sentinel failover; local in-process bucket as soft fallback |
| RTO | ≤ 30 s sentinel failover |
| Recovery runbook | inherited from `cell/runbooks/redis-failover.md` |
| Postmortem owner | ops-sre-reliability |

## FM-FP-12: Audit-chain emission backlog

| Field | Value |
|---|---|
| Trigger | `foundry-evidence` µservice slow consuming `ProviderInvoked` events; NATS JetStream queue grows |
| Detection | `oya_foundry_providers_audit_emission_queue_depth` exceeds threshold |
| Tenant impact | Tenant calls still succeed; audit-chain seal latency grows |
| Severity | Sev-3 if queue depth manageable; Sev-2 if seal latency SLO breached |
| Immediate mitigation | Scale `foundry-evidence` consumers; backpressure protocol activates |
| RTO | ≤ 15 min queue drain |
| Recovery runbook | inherited from `foundry-evidence/runbooks/audit-emission-backlog.md` |
| Postmortem owner | ops-sre-reliability + axis-foundry |

## FM-FP-13: Tool-call exfil attempt

| Field | Value |
|---|---|
| Trigger | Vendor response includes a tool-use proposal that, if executed, would exfil tenant data |
| Detection | Cedar policy `cell/policy/tool-execution.cedar` deny event; pattern detector at `cell` |
| Tenant impact | Tool call refused; tenant sees deterministic deny; no data exfil |
| Severity | Sev-2 (security event; investigate) |
| Immediate mitigation | Cedar deny is the immediate mitigation; per `threat-model.md` T-07, adapter does not execute tool calls |
| RTO | n/a (event prevented) |
| Recovery runbook | ops-security incident playbook |
| Postmortem owner | ops-security |

## FM-FP-14: Cost-ceiling breach (single tenant)

| Field | Value |
|---|---|
| Trigger | Per-tenant per-day cost exceeds configured ceiling |
| Detection | `oya_foundry_providers_provider_cost_usd_total{tenant="<t>"}[1d]` exceeds ceiling |
| Tenant impact | Router stops invocations for that tenant until ceiling raised or next-day reset |
| Severity | Sev-3 (planned ceiling enforcement) / Sev-2 (unexpected ceiling breach indicates anomalous workload) |
| Immediate mitigation | Tenant outreach; ceiling raise OR workload review |
| RTO | per ops-finops + tenant operator decision |
| Recovery runbook | `runbooks/rate-limit-cascade-recovery.md` §"Cost-ceiling breach" |
| Postmortem owner | ops-finops + axis-foundry |

## FM-FP-15: In-house GPU node failure

| Field | Value |
|---|---|
| Trigger | GPU node hardware failure |
| Detection | k8s node `Ready=False`; in-house adapter sees increased timeouts |
| Tenant impact | Router auto-demotes in-house; routes to alternate vendor |
| Severity | Sev-3 if redundancy absorbs; Sev-2 if pool capacity insufficient |
| Immediate mitigation | k8s replaces node; in-house adapter recovers |
| RTO | ≤ 10 min node replace |
| Recovery runbook | inherited from `cloud-k8s/runbooks/gpu-node-failure.md` |
| Postmortem owner | ops-sre-reliability + axis-foundry |

## Acceptance

- All failure modes mapped to a runbook (or cross-µservice inherited runbook).
- All failure modes have detection signal in Mimir.
- Quarterly drill of FM-FP-01 + FM-FP-03 + FM-FP-04 + FM-FP-05; drill evidence in `evidence/runbook-drills/`.
- Per-Sev-1 incident: postmortem published; failure-mode catalog updated if a new pattern emerged.

## References

- `microservices/foundry-providers/threat-model.md`.
- `microservices/foundry-providers/dpia.md`.
- `microservices/foundry-providers/incident-response.md`.
- `microservices/foundry-providers/runbooks/`.
