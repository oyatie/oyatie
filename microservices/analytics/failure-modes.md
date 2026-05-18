# Analytics µservice — Failure Modes and Effects Analysis (FMEA)

**Authority:** ADR-0193, ADR-0001 cohesion, hyperscaler-architecture-invariants.json (4-INV)
**Owner:** council-analytics + ops-sre-reliability
**Last reviewed:** 2026-05-18

This document enumerates each component, its credible failure modes, the detection path, the immediate mitigation, the long-term mitigation, and the residual risk after both. Where a failure cuts across multiple components, it appears under the root component with cross-references.

## 1. ClickHouse server replica

### 1.1 Process crash (panic, OOM, segfault)

- **Detection:** Kubernetes liveness probe + `ClickHouseInstanceDown` alert (IP-001).
- **Immediate mitigation:** Kubernetes restarts the pod; another replica in the shard serves queries; no client-visible impact for shards with replication-factor ≥ 2.
- **Long-term mitigation:** Memory limits enforced via Helm values; OOM-killer threshold above `max_server_memory_usage`; panic captured in core dump uploaded to `evidence/core-dumps/`.
- **Residual risk:** Negligible (replica-level).

### 1.2 Slow disk → query latency spike

- **Detection:** `ClickHouseProfileEvents_DiskReadElapsedMicroseconds` p99 alert in PrometheusRule.
- **Immediate mitigation:** Shed load via `max_concurrent_queries` reduction; investigate underlying disk via `kubectl describe pod`; check CSI fast-class node health.
- **Long-term mitigation:** Node label `oya.disk-class=nvme-premium` enforced at scheduling; HW vendor SLA escalation if pattern persists.
- **Residual risk:** Low.

### 1.3 Replica out of sync (replication queue backlog)

- **Detection:** `system.replication_queue` row count > threshold; `ClickHouseReplicationLag > 60s` alert.
- **Immediate mitigation:** Investigate per `runbooks/clickhouse.md` §"Replication lag"; restart the lagging replica if stuck; possible `SYSTEM RESTART REPLICA` operation.
- **Long-term mitigation:** Increased replica disk throughput; partition strategy review if specific partitions repeatedly stall.
- **Residual risk:** Low.

## 2. ClickHouse Keeper

### 2.1 Single Keeper failure

- **Detection:** Pod NotReady; Raft cluster reports 2-of-3.
- **Immediate mitigation:** Kubernetes restarts the pod; quorum maintained at 2-of-3; cluster DDL continues.
- **Long-term mitigation:** Anti-affinity ensures Keeper pods land on distinct nodes; node anti-affinity at zone level for multi-AZ cells.
- **Residual risk:** Negligible.

### 2.2 Quorum loss (2 or more Keepers down)

- **Detection:** `ClickHouseKeeperNoLeader` alert + DDL errors at the application layer.
- **Immediate mitigation:** Read traffic continues (no DDL needed); follow `runbooks/keeper-quorum-recovery.md`; restore from Keeper snapshot if needed.
- **Long-term mitigation:** Production cells use 5-replica Keeper quorum (overlay in pack-eu, pack-kr).
- **Residual risk:** Low at 5-replica; medium at 3-replica.

### 2.3 Keeper snapshot corruption

- **Detection:** Keeper restart reports snapshot load failure.
- **Immediate mitigation:** Restore from prior snapshot (snapshots are written every 100K log entries; previous snapshot is at most ~10min old).
- **Long-term mitigation:** Snapshot verification job runs nightly; corrupted snapshots quarantined.
- **Residual risk:** Low.

## 3. Kafka engine / Pulsar KoP

### 3.1 Pulsar broker outage (single)

- **Detection:** Consumer lag spike; Pulsar HA topology routes to surviving broker.
- **Immediate mitigation:** Pulsar HA (3 brokers) handles automatically; no action required.
- **Residual risk:** Negligible.

### 3.2 Pulsar broker total outage

- **Detection:** Consumer offset reports no progress; ingest pipeline halted.
- **Immediate mitigation:** Customer-visible — dashboards stale beyond 5s SLO. Restore Pulsar; consumer resumes from last committed offset; backlog drains.
- **Long-term mitigation:** Cell-local Pulsar HA; cross-cell Pulsar federation for global tenants.
- **Residual risk:** Low (Pulsar HA covers single-AZ failure; multi-AZ outage is a wider event).

### 3.3 Consumer offset corruption

- **Detection:** Consumer reports invalid offset; lag goes negative or to ∞.
- **Immediate mitigation:** Reset offset to last known good via `clickhouse-client -q 'SYSTEM RESTART CONSUMER ...'`; possibly replay from earliest available offset.
- **Long-term mitigation:** Offset commits are durable in Keeper; periodic offset snapshot to S3.
- **Residual risk:** Low.

### 3.4 Schema drift between source µservice and Kafka engine table

- **Detection:** MV insertion errors; rows with unexpected fields dropped.
- **Immediate mitigation:** Roll back the source µservice schema change OR add the new column to the Kafka engine table (online ALTER).
- **Long-term mitigation:** ADR-0154 event schema versioning enforces backward-compatible schema evolution at CI lane.
- **Residual risk:** Low.

## 4. Materialized View target tables

### 4.1 MV freeze (target table merge backlog)

- **Detection:** `system.merges` length > threshold; `ClickHouseMetrics_PartsDelayInsert > 0`.
- **Immediate mitigation:** Slow upstream ingest (Kafka engine backpressure); investigate partition strategy.
- **Long-term mitigation:** Partition cardinality review; consider monthly→weekly partitions for hot tables.
- **Residual risk:** Low.

### 4.2 MV produces wrong aggregation (bug)

- **Detection:** Reconciliation lane (IP-014) compares MV output against source-of-truth periodic recompute.
- **Immediate mitigation:** Drop the MV; recompute from source via `backfill-replay.md`.
- **Long-term mitigation:** MV change review checklist; canary deploy.
- **Residual risk:** Medium (logic bugs).

### 4.3 MV target table missing (tenant onboarded but MV not applied)

- **Detection:** Insertion error log; controller cursor lag.
- **Immediate mitigation:** IP-002 controller re-reconciles.
- **Long-term mitigation:** Reconciliation job hourly checks for missing per-tenant MVs.
- **Residual risk:** Low.

## 5. Per-tenant database bootstrap controller

### 5.1 Controller crash mid-onboard

- **Detection:** State cursor lag on restart.
- **Immediate mitigation:** Controller re-reads cursor and reconciles; idempotent operations re-apply cleanly.
- **Long-term mitigation:** Controller leader election (2 replicas, one active).
- **Residual risk:** Negligible.

### 5.2 Quota mis-application (wrong tier applied)

- **Detection:** Reconciliation lane compares observed vs desired quota.
- **Immediate mitigation:** Controller re-reads tenancy state and re-applies.
- **Long-term mitigation:** Tier change events are idempotent.
- **Residual risk:** Low.

### 5.3 Tenant offboard partial completion (DB dropped, proof-of-erasure not emitted)

- **Detection:** Audit-chain reconciliation; proof-of-erasure missing within SLA.
- **Immediate mitigation:** Controller re-emits proof-of-erasure event with corrected timestamp.
- **Long-term mitigation:** Two-phase commit: emit proof-of-erasure to outbox BEFORE drop; outbox publisher ensures eventual delivery.
- **Residual risk:** Low.

## 6. Cold-tier S3

### 6.1 S3-compat (SeaweedFS) outage

- **Detection:** S3 5xx rate alert; cold-tier query latency p99 spike.
- **Immediate mitigation:** Notify customers ("queries on data older than 90 d may be slow"); hot tier unaffected; per-cell SeaweedFS replication recovers.
- **Long-term mitigation:** Multi-AZ SeaweedFS deployment per cell; cross-cell failover for non-residency-strict tenants.
- **Residual risk:** Low.

### 6.2 TTL→TODISK stuck (parts not migrating)

- **Detection:** `system.parts.disk_name` shows hot-tier parts past TTL.
- **Immediate mitigation:** `SYSTEM TTL ON CLUSTER ...` to force a TTL pass.
- **Long-term mitigation:** Monitoring on TTL pass timing.
- **Residual risk:** Negligible.

### 6.3 Cold-tier read very slow (>10s p99)

- **Detection:** Cold-tier SLO burn alert.
- **Immediate mitigation:** Customer notice; investigate S3 health, network path.
- **Long-term mitigation:** Dashboard-layer caching for cold-tier reads; predictive prefetch.
- **Residual risk:** Medium (inherent to S3 latency).

## 7. Dashboard API (IP-007)

### 7.1 Cedar policy bug → spurious 403

- **Detection:** 403 rate spike alert.
- **Immediate mitigation:** Roll back the Cedar policy change.
- **Long-term mitigation:** Cedar policy CI lane verifies "principal X cannot access tenant Y's data" against test fixture.
- **Residual risk:** Low.

### 7.2 Cursor signature replay attack

- **Detection:** HMAC verification fails on tampered cursor.
- **Immediate mitigation:** API returns 400 + audit event.
- **Long-term mitigation:** Cursor TTL (1h); rotate HMAC signing key per ADR-0157.
- **Residual risk:** Low.

### 7.3 Large response → connection timeout

- **Detection:** 502 / 504 alert.
- **Immediate mitigation:** Enforce max page_size at the API layer; chunked transfer encoding for legitimate large responses (regulator export).
- **Long-term mitigation:** Streaming JSON for unbounded result sets.
- **Residual risk:** Low.

## 8. Audit-log query API (IP-008)

### 8.1 Cold-tier query > 2s p99

- **Detection:** SLO burn (IP-014).
- **Immediate mitigation:** Customer notice; investigate cold-tier health.
- **Long-term mitigation:** Hot-tier extension for "last 365 d" queries on Enterprise tier.
- **Residual risk:** Medium (cold-tier inherent latency).

### 8.2 Recursive audit-event storm (querying audit log emits audit event ⇒ inf loop)

- **Detection:** Audit-chain rate-of-emission alert.
- **Immediate mitigation:** Special-case: queries against the audit-event table itself emit a single meta-event per minute, not per query.
- **Long-term mitigation:** Implemented in IP-008 from the start.
- **Residual risk:** Negligible.

## 9. Billing rollup pipeline (IP-009)

### 9.1 Daily rollup ≠ monthly rollup (reconciliation drift)

- **Detection:** Month-end reconciliation lane.
- **Immediate mitigation:** Investigate; if MV bug, recompute from source.
- **Long-term mitigation:** Daily reconciliation, not just monthly.
- **Residual risk:** Low (acceptable drift < 0.01%).

### 9.2 Out-of-order usage events → wrong daily attribution

- **Detection:** Late-arriving event detection (event_time < ingest_time - 2h).
- **Immediate mitigation:** Adjust to event_time, not ingest_time, in the MV GROUP BY.
- **Long-term mitigation:** Already implemented via `toDate(emitted_at)` in MV.
- **Residual risk:** Negligible.

## 10. Cross-cell federation (IP-010)

### 10.1 Cross-cell `remote()` from tenant principal (policy bypass attempt)

- **Detection:** Cedar denies; audit event.
- **Mitigation:** Already foreclosed at policy layer.
- **Residual risk:** Negligible.

### 10.2 Cross-cell DDL drift (Distributed table not on every cell)

- **Detection:** Schema-reconciliation lane.
- **Immediate mitigation:** Re-deploy IaC; verify per-cell consistency.
- **Long-term mitigation:** GitOps reconcile loop.
- **Residual risk:** Low.

## 11. Backup pipeline (IP-012)

### 11.1 Daily backup window overrun

- **Detection:** Backup job > 4h.
- **Immediate mitigation:** Split into per-tenant backups; parallelize.
- **Long-term mitigation:** Sharded backup per top-N tenants.
- **Residual risk:** Low.

### 11.2 Backup signature failure

- **Detection:** cosign verify fails on pull.
- **Mitigation:** See incident-response.md §5.4.
- **Residual risk:** Low.

## 12. Self-SLO authoring (IP-014)

### 12.1 SLO source compiles but PrometheusRule wrong

- **Detection:** Alert never fires for genuine burn.
- **Immediate mitigation:** Inject synthetic burn; verify alert path; correct PrometheusRule.
- **Long-term mitigation:** Synthetic burn test in CI (sloth-compile-test).
- **Residual risk:** Low.

## 13. Composite failure modes

### 13.1 Keeper quorum loss + ingest pipeline outage simultaneously

- **Detection:** Two Sev 1s declared.
- **Mitigation:** IC chooses higher priority (typically Keeper since it blocks DDL); CL communicates both impacts.
- **Residual risk:** Medium (composite failure rare; mitigation procedural).

### 13.2 Cross-tenant leak + backup compromise (worst-case)

- **Detection:** Both triggers fire.
- **Mitigation:** Sev 1 + Sev 1; GDPR 72h triggered; forensic + legal hold.
- **Residual risk:** Low (each independently rare; conjunction extremely rare).

## 14. References

- ADR-0193 (engine choice), ADR-0001 cohesion, ADR-0152 RPO/RTO, ADR-0180-dr-business-continuity-portfolio-policy, ADR-0003 audit chain.
- specs/hyperscaler-architecture-invariants.json (4-INV).
- NIST SP 800-30 Rev. 1 (risk assessment).
