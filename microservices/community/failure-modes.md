---
doc_class: FailureModes
template_id: TPL-FAILURE-MODES
microservice: community
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-community + ops-sre
related_adrs: [ADR-0056, ADR-0105, ADR-0126, ADR-0131]
related_artifacts:
  - microservices/community/incident-response.md
  - microservices/community/runbooks/
  - microservices/community/threat-model.md
doc_status: published
---

# Failure Modes: community µservice

## Catalogue

| # | Failure Mode | Trigger | Blast Radius | Detection | Mitigation | Runbook | RTO | RPO |
|---|---|---|---|---|---|---|---|---|
| FM-01 | Search index rebuild storm | Cascading per-tenant reindex + new schema deploy | All search traffic in cluster | Search p99 > 1.5 s for 5 min | Per-tenant token-bucket; staggered rebuild windows | runbooks/search-rebuild.md | 60 min | 0 (rebuild from source) |
| FM-02 | Vote race / double-count | Two replicas write same vote simultaneously | Vote tally divergence on one post | Vote tally drift > 0.1 % | Redis Lua atomic increment; idempotency key per (member, post) | runbooks/vote-anomaly.md | 5 min | 0 |
| FM-03 | Moderation queue OOM | Coordinated flag campaign | Tenant's moderation processing halted | Worker OOM kill + queue depth > 100 k | Per-tenant queue cap; overflow to S3 cold queue | runbooks/moderation-queue-clear.md | 30 min | 0 |
| FM-04 | KB attachment store outage (S3 region) | Cloud provider regional event | KB attachment uploads + reads fail | S3 PUT/GET p99 > 5 s; error rate > 1 % | Cross-region replication; degraded mode (show without attachment) | runbooks/kb-attachment-restore.md | 4 h | 5 min |
| FM-05 | Mass-spam abuse from compromised member | Stolen credential bot | Tenant feeds polluted | Post velocity z-score > 5 | Per-member rate-limit; foundry-guardrails fast-path block; member quarantine | runbooks/spam-flood-throttle.md | 5 min | 0 |
| FM-06 | Postgres primary failover (Patroni) | Node hardware / network event | Write path stall | Patroni leader election | Patroni auto-failover; replica promoted; WAL replay | runbooks/postgres-failover.md (in tenancy) | 60 s | 30 s |
| FM-07 | Post mass-deletion (compromised tenant_admin) | Stolen tenant_admin credential | Tenant's posts deleted | Audit-chain delete-rate anomaly | Two-eyes on `delete_post` over 100/day threshold; restore from WAL | runbooks/post-mass-deletion.md | 30 min | 30 s |
| FM-08 | Elasticsearch shard corruption | Disk error / silent corruption | Tenant search returns stale / wrong results | Index health check fail | Replica promotion; reindex from Postgres source-of-truth | runbooks/search-rebuild.md | 60 min | 0 |
| FM-09 | Redis hot-feed cache stampede | Trending post causes thundering-herd | Tenant feed latency spike | Postgres feed-fill rate > 100×; Redis CPU > 90 % | Per-tenant Redis namespace memory quota; single-flight in adapter; LFU eviction | runbooks/spam-flood-throttle.md | 5 min | 0 |
| FM-10 | ClamAV scanner failure | Signature DB outdated / scanner OOM | KB attachment uploads queue | Scan latency > 30 s; queue depth > 1 k | Fallback scanner replica; reject upload on extended outage; retry queue | runbooks/kb-attachment-restore.md | 30 min | 0 |
| FM-11 | Foundry-guardrails bridge backpressure | Classifier model down / slow | Spam detection lag → feeds pollute | Bridge p99 > 10 s; lag > 100 k events | Bridge dead-letter queue; rate-limit fallback; manual moderation | runbooks/spam-flood-throttle.md | 15 min | 5 min |
| FM-12 | Audit-chain seal lag | audit-chain µservice degraded | Seal latency p99 > 5 s | Audit-chain seal queue depth > 10 k | Bridge buffers events; tenant write-path stays open; seal catches up | (audit-chain incident-response.md) | 30 min | 5 min |
| FM-13 | Mention-resolution table out-of-sync with tenancy | Tenancy member-update lost / delayed | Mentions return unresolved | Mention unresolved rate > 1 % | Replay from tenancy events; reconcile job | runbooks/mention-reconcile.md | 30 min | 5 min |
| FM-14 | KB article impersonation (stolen credential) | Compromised author credential | Tenant trust in KB articles | Author velocity anomaly | foundry-guardrails detector; revoke session; revert via revision | runbooks/post-mass-deletion.md | 15 min | 0 |
| FM-15 | Cross-tenant search bleed (policy bug) | Cedar fragment regression | Cross-tenant data leak | Cedar deny-rate anomaly inverted | Immediate gateway disable; revert deploy; security incident | runbooks/cross-tenant-bleed.md | 5 min | 0 |
| FM-16 | Cedar policy compilation failure | Bad fragment merge | All writes denied | Cedar compile error in startup | Block deploy in CI; rollback to prior policy hash | runbooks/cross-tenant-bleed.md | 10 min | 0 |
| FM-17 | NATS event bus outage | Bus down | Foundry-guardrails + audit-chain integration stall | NATS health probe fail | NATS JetStream; redundant cluster; bridge buffers | (cloud-iac incident-response.md) | 10 min | 5 min |
| FM-18 | OpenBao secret rotation drift | Rotation missed | Connection failure to ES / Redis / S3 | Connection error rate spike | Pre-rotation grace period; alert on cert age > 30 d | (cloud-secrets incident-response.md) | 30 min | 0 |
| FM-19 | Worker pool fair-share starvation | One noisy tenant | Other tenants' workers starve | Per-tenant token-bucket fill ratio < 10 % | Per-tenant token bucket; QoS prioritisation; noisy-neighbour eviction | runbooks/spam-flood-throttle.md | 15 min | 0 |
| FM-20 | DSR Right-to-Erasure cascade fails partway | One downstream µservice slow | Audit gap; tenant SLA risk | DSR job completion timeout | Resumable cascade; per-step idempotency; manual rerun | runbooks/dsr-cascade-resume.md | 30 d (per regulation) | 0 |

## Severity Mapping

- **P0 (≤5 min response)**: FM-04 partial, FM-05, FM-07, FM-15, FM-16
- **P1 (≤15 min response)**: FM-01, FM-02, FM-03, FM-08, FM-09, FM-11, FM-12, FM-14, FM-17, FM-19
- **P2 (≤30 min response)**: FM-06 (covered by tenancy), FM-10, FM-13, FM-18
- **P3 (next business day)**: FM-20

## Tabletop Cadence

- Quarterly: rotate through FM-01, FM-03, FM-05, FM-07, FM-15.
- Annually: full chaos drill covering every P0 + P1.
