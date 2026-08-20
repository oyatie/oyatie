---
doc_class: Runbook
title: Valkey cluster failover (partition / ACL drift / shard fail)
microservice: foundry-runtime
severity: "Sev-1 (ACL drift; security risk) / Sev-2 (cluster partition or shard fail)"
status: Accepted
owner_team: ops-sre-reliability + axis-foundry-runtime
date: 2026-05-17
related_artifacts:
  - microservices/intelligence/failure-modes.md (FM-02, FM-03)
  - microservices/intelligence/threat-model.md (T-T-02)
  - microservices/intelligence/policy/runtime-isolation.md
  - microservices/intelligence/multi-region.md
doc_status: published
---

# Runbook: Valkey cluster failover

## Trigger

ONE of:
- Valkey cluster partition (FM-02): `oya_foundry_runtime_redis_connection_failures_total > threshold`; majority loss in shards.
- Valkey ACL drift (FM-03): `oya-check-session-prefix-isolation` lane fails OR Helm-state-validator alarms; OR `oya_foundry_runtime_unauthorized_attempt_total > 0` from Redis-side rejection.
- Single shard fail: primary unreachable; replica must promote.

## Severity

- Partition / shard fail: Sev-2.
- ACL drift: Sev-1 (security risk; potential cross-tenant exposure).

## Cluster partition (FM-02)

| Step | Action | Time |
|---|---|---|
| 1 | Identify affected shards: `redis-cli -h <node> cluster info` + `cluster nodes` | ≤2min |
| 2 | Verify replica promotion: for each shard with majority loss, replica auto-promotes via Valkey Sentinel / Cluster | ≤5min |
| 3 | Check ingest queue depth in runtime executor pods: backpressure to clients via 503 (not infinite queue) | ≤2min |
| 4 | HPA may scale runtime-pool to absorb cold-restore latency hit (Valkey miss → Postgres) | ≤5min |
| 5 | Cordon affected AZ if pattern: `kubectl cordon <node>` | ≤5min |
| 6 | Once primary side network restored: rejoin nodes; verify cluster health: `redis-cli cluster info` returns `cluster_state:ok` | ≤15min |
| 7 | Verify recovery: `redis_connection_failures_total` rate back to baseline | ≤5min |
| 8 | If outage > 30min: consider DR failover per `multi-region.md` | ≤35min |

## ACL drift (FM-03)

| Step | Action | Time |
|---|---|---|
| 1 | Engage Sev-1; open `#inc-sec-<id>` Slack; declare ops-security | immediate |
| 2 | Snapshot current ACL: `redis-cli ACL LIST > /tmp/acl-current.txt` | ≤2min |
| 3 | Compare with declared (Helm values): `diff <(redis-cli ACL LIST) microservices/intelligence/iac/helm/redis/expected-acl.txt` | ≤2min |
| 4 | Auto-rollback via ArgoCD: apply declared Helm values; live ACL reconciled | ≤5min |
| 5 | Verify ACL restored: every tenant role has prefix-scoped command set; `default` user disabled | ≤2min |
| 6 | Audit who mutated ACL: OpenBao audit log + Valkey audit log + Kubernetes audit log | – |
| 7 | If exposure occurred during drift window (`oya_foundry_runtime_unauthorized_attempt_total > 0` between drift and rollback): begin breach-notification chain per `incident-response.md` §"Regulatory Notifications" | per pack |
| 8 | Postmortem: was CI lane bypassed? was the live-cluster admin path open? | within 5 business days |

## Single shard fail (sub-case of FM-02)

| Step | Action | Time |
|---|---|---|
| 1 | Identify failing shard: `redis-cli cluster info` shows failed primary | ≤2min |
| 2 | Replica auto-promote (Valkey Cluster); verify promotion: `cluster nodes` | ≤2min |
| 3 | Schedule replacement of failed pod; `kubectl rollout restart sts/oya-intelligence-runtime-redis-<shard>` | ≤10min |
| 4 | Once new pod up, verify it joins as replica and replication starts | ≤5min |

## Verification

After recovery:
- `redis-cli cluster info` returns `cluster_state:ok` + all shards reporting; replicas synced.
- `redis_connection_failures_total` rate back to baseline.
- Session hot reads p99 ≤10ms restored.
- For ACL drift: ACL matches expected exactly; no `oya_foundry_runtime_unauthorized_attempt_total > 0` after rollback.
- Self-observability dashboard green.

## Post-incident updates

- Postmortem within 5 business days.
- For FM-02 repeated: investigate underlying instance / network reliability; consider higher replication factor.
- For FM-03: harden control-plane access to Valkey (Kubernetes RBAC + OpenBao JIT-only); audit which path allowed live mutation; tighten admission controllers.

## References

- `microservices/intelligence/failure-modes.md` FM-02, FM-03.
- `microservices/intelligence/threat-model.md` T-T-02.
- `microservices/intelligence/policy/runtime-isolation.md` TI-01, TI-02.
- `microservices/intelligence/multi-region.md` §"DR Failover".
- Valkey 8.1 (Redis wire-compat) — `redis.io/docs/about/releases/7-4-0/`.
- Valkey Cluster — `redis.io/docs/management/scaling/`.
- Valkey ACL — `redis.io/docs/management/security/acl/`.
