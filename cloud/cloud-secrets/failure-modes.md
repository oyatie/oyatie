---
doc_class: FailureModes
microservice: cloud-secrets
status: Accepted
date: 2026-05-17
owner_team: axis-cloud-secrets + ops-sre + ops-security
related_adrs: [ADR-0028, ADR-0117, ADR-0131]
related_artifacts:
  - microservices/cloud-secrets/threat-model.md
  - microservices/cloud-secrets/incident-response.md
  - microservices/cloud-secrets/runbooks/*.md
review_cadence: quarterly + post every Sev-1/Sev-2
doc_status: published
---

# Failure Modes: cloud-secrets µservice

This document enumerates the failure modes the cloud-secrets substrate can experience, their detection signal, their blast radius, their mitigation/recovery path, and the runbook owner.

## Severity Ladder

| Sev | Definition | Notification target |
|---|---|---|
| Sev-1 | Secret-resolution unavailable cluster-wide OR raw-secret-leak detected OR HSM compromise | grafana-oncall page; ops-security + axis-cloud-secrets + on-call exec; tenant notification within 72h (Art. 33) |
| Sev-2 | Pack-scoped degradation; SLO error budget burn > 14.4× over 1h | grafana-oncall page; axis-cloud-secrets + ops-sre |
| Sev-3 | Tenant-scoped or microservice-scoped degradation | ticket; axis-cloud-secrets |
| Sev-4 | Cosmetic, dashboard, latency hiccup within budget | observability lane |

## FM-01 — OpenBao cluster Raft quorum loss

- **Description**: ≥3 of 5 Raft peers unreachable; cluster cannot accept writes; reads continue from quorum survivors briefly until lease expiry.
- **Trigger**: node failure, network partition, Postgres backend outage, OOMKill cascade.
- **Detection**: OpenBao `vault_core_unsealed=0` on >2 peers; `up{job="openbao"}` failing on >2; leader-election storm metric.
- **Blast radius**: pack-wide secret resolution + rotation blocked (Sev-1).
- **Mitigation**:
  - Cache TTL ≤60s permits consumer SDKs to serve stale-but-valid for up to 60s.
  - openbao-operator auto-restart of peers; manual quorum-recovery via `bao operator raft remove-peer` if persistent.
  - Postgres-HA failover (Patroni) decouples backend availability.
- **Recovery**:
  - openbao-operator's HA reconciler restarts unhealthy pods.
  - If Postgres backend lost: Patroni promotes replica.
  - If quorum cannot be recovered: invoke `runbooks/openbao-restart.md` Sev-1 path.
- **RPO/RTO**: RPO ≤1s (Raft log replication lag); RTO ≤2 min.
- **Runbook**: `runbooks/openbao-restart.md`.

## FM-02 — HSM partition unavailable

- **Description**: OCI Cloud-HSM or Thales Luna partition unreachable; KEK signing operations fail; auto-unseal fails on OpenBao restart.
- **Trigger**: HSM hardware failure, vendor maintenance window, network path failure, attestation revocation.
- **Detection**: PKCS#11 client error rate > 1% over 60s; HSM heartbeat probe timeout.
- **Blast radius**: rotation operations blocked; OpenBao continues serving existing KV (KEK already unwrapped in memory) (Sev-2). On restart, HSM unavailability blocks unseal (Sev-1).
- **Mitigation**:
  - HSM HA replica (each pack has 2 partitions); failover via PKCS#11 client retry.
  - Pre-fetched DEK cache reduces in-band KEK ops.
  - HSM vendor SLA (OCI 99.95%, Thales 99.95%).
- **Recovery**:
  - Auto-failover to HA partition.
  - If both partitions down: pack-wide Sev-1; runbook `runbooks/hsm-key-rotation.md` §"HSM compromise / unavailability".
- **RPO/RTO**: RPO 0 (KEK is HSM-resident; not replicated cross-partition; failover is to HA partition with same KEK reference); RTO ≤5 min.
- **Runbook**: `runbooks/hsm-key-rotation.md`.

## FM-03 — Key-rotation stuck (scheduler hung or rotation failure)

- **Description**: A scheduled rotation does not complete within its SLA window (60s for most rotations); subsequent rotations queue.
- **Trigger**: HSM unavailable, OpenBao policy mis-author, cascade-dependency deadlock, scheduler bug.
- **Detection**: `oya_cloud_secrets_rotation_overdue_total > 0` over 5min; `RotationOverdue` event.
- **Blast radius**: affected secret approaches expiry; if expiry passes, downstream µservices fail (Sev-2).
- **Mitigation**:
  - Scheduler retry with exponential backoff (3 attempts, then page).
  - Cascade-rotation: dependents pause until parent rotates.
  - Rotation SLA is conservative (KEK 365d, signing keys 90d): plenty of room before secret-expiry.
- **Recovery**:
  - Examine `RotationOverdue` payload; manual rotation via `openbao kv rotate <path>` after diagnosis.
  - If cascade deadlock: invoke `runbooks/rotation-cascade-recovery.md`.
- **RPO/RTO**: RPO n/a (rotation state derivable from OpenBao); RTO ≤30 min.
- **Runbook**: `runbooks/rotation-cascade-recovery.md`.

## FM-04 — Audit-emission backpressure stalls SecretAccessed events

- **Description**: `audit-emitter → audit-chain` bridge throughput drops; events queue locally; local disk fills.
- **Trigger**: audit-chain µservice degradation; network path failure; bridge crash loop.
- **Detection**: `oya_cloud_secrets_audit_emission_backlog_seconds > 60`; local audit-device file growth rate.
- **Blast radius**: secret-resolution continues; audit lag breaches compliance SLA (PIPA Art. 29 + SOC 2 CC7.1); regulatory Sev-2.
- **Mitigation**:
  - Local audit-device file is durable + capped (rotate at 10 GiB).
  - audit-emitter retries with exponential backoff.
  - Backlog > 60s pages; > 300s escalates Sev-1.
- **Recovery**:
  - Diagnose audit-chain; restart bridge worker.
  - Bulk-replay from local audit-device file.
  - If audit-chain unrecoverable: invoke audit-chain µservice's own incident path.
- **RPO/RTO**: RPO ≤1s (local audit-device); RTO ≤15 min.
- **Runbook**: `runbooks/audit-emission-backlog.md`.

## FM-05 — Per-tenant namespace controller crash

- **Description**: Controller pods crash-loop or fall behind; new `TenantRegistered` events accumulate in queue; tenant onboarding stalls.
- **Trigger**: bad config push, RBAC mis-author, OpenBao API error cascade, Kubernetes lease contention.
- **Detection**: `oya_cloud_secrets_namespace_provisioning_lag_seconds > 600`; pod restart counter; `NamespaceProvisioningStuck` event.
- **Blast radius**: tenant onboarding blocked (Sev-2); existing tenants unaffected.
- **Mitigation**:
  - 2+ replicas with Kubernetes lease leader-election.
  - Reconciliation is idempotent; restart resumes.
  - Backlog metric drives HPA scale-up.
- **Recovery**:
  - Inspect logs; restart pods; if persistent crash-loop, rollback to last-known-good controller image.
- **RPO/RTO**: RPO 0 (state in OpenBao + Kubernetes API); RTO ≤10 min.
- **Runbook**: `runbooks/namespace-controller-restart.md`.

## FM-06 — Raw-secret-leak detected in repo/chat/checkpoint

- **Description**: LEAN-A11 lane or quarterly retroactive scan or external report identifies a raw secret in the repo, agent chat transcript, or `.omc/state/` checkpoint.
- **Trigger**: agent or operator inadvertently emits raw value bypassing controls; tool-output insufficiently redacted; new pattern not yet in scanner.
- **Detection**: LEAN-A11 lane fail (BLOCKER on PR); scanner finding; external responsible-disclosure.
- **Blast radius**: secret may be public; assume compromise; immediate revoke + rotate cascade (Sev-1 ALWAYS).
- **Mitigation**:
  - LEAN-A11 BLOCKER prevents most leaks pre-merge.
  - Per `Secret<T>` newtype + scrubbed logs reduces emission surface.
- **Recovery**:
  - Sev-1 incident: revoke secret immediately (≤5s); cascade-rotate dependents.
  - Git history rewrite is futile (assume already cloned/cached); rely on revocation.
  - Forensic: identify how the leak occurred; tighten pattern; post-mortem.
  - Tenant notification within 72h (GDPR Art. 33), 24h (LGPD Art. 48), per pack legal SLA.
- **RPO/RTO**: RPO 0 (revocation immediate); RTO ≤5min (cascade complete).
- **Runbook**: `runbooks/secret-leak-detected.md` (Sev-1 always).

## FM-07 — Resolver SDK in-process cache poisoning

- **Description**: A consumer µservice's in-process cache holds a corrupted or stale value (manifests as application-level failure).
- **Trigger**: process-memory-write attacker (already privileged), bug in SDK serde, lifecycle race.
- **Detection**: consumer-side application error rate spike correlated with cache TTL boundary.
- **Blast radius**: per-process; isolated (Sev-3).
- **Mitigation**:
  - Cache values held in `zeroize::Zeroizing<String>`; cache key includes HMAC.
  - TTL ≤60s bounds stale-window.
  - SDK validates value via HMAC on every cache-hit.
- **Recovery**:
  - Restart affected consumer pod; cache is in-process and ephemeral.
  - If pattern persists: SDK bug; halt deploy; investigate.
- **RPO/RTO**: RPO 0; RTO seconds.
- **Runbook**: covered by general µservice incident runbook (no cloud-secrets-specific path needed).

## FM-08 — Revocation push not propagating

- **Description**: `SecretRevoked` event emitted; one or more consumer SDKs do not flush cache; consumers continue serving stale (revoked) value.
- **Trigger**: SSE/WebSocket connection dropped + reconnect failure, consumer SDK bug, network partition.
- **Detection**: `oya_cloud_secrets_revocation_propagation_lag_seconds > 5`; consumer-side cache-hit on revoked path.
- **Blast radius**: leaked credential remains usable longer than SLA; security Sev-1 if revocation was triggered by suspected compromise.
- **Mitigation**:
  - SDK opens persistent SSE; auto-reconnect with backoff.
  - On reconnect, SDK queries `/revoked-since=<last-seen-id>` to catch up.
  - OpenBao policy "revoke-on-fetch": every resolve double-checks revocation list (fast bloom-filter).
- **Recovery**:
  - Identify lagging consumers; force cache flush via SDK admin endpoint or pod restart.
- **RPO/RTO**: RPO ≤5s; RTO ≤30s.
- **Runbook**: `runbooks/secret-leak-detected.md` §"Revocation cascade".

## FM-09 — Cross-pack secret-replication attempt (residency breach)

- **Description**: A workflow or operator attempts to write a secret created in pack-A into pack-B's OpenBao (e.g., via a misconfigured encryption-key BYOK upload routing, ADR-0251 §D-10).
- **Trigger**: pack-routing.cedar mis-author, operator mistake, malicious insider.
- **Detection**: Cedar deny on `pack-routing.cedar`; audit-emit `cross_pack_write_attempt`; alert.
- **Blast radius**: residency contract breach — would be regulatory Sev-1 if successful; intercepted at policy layer makes it Sev-2.
- **Mitigation**:
  - Default-deny Cedar policy.
  - Per-pack OpenBao endpoint isolation (different DNS + network policy).
  - Audit-emit on every attempt.
- **Recovery**:
  - Block attempt at policy; investigate root cause; tighten Cedar fragment if needed.
- **RPO/RTO**: RPO 0; RTO immediate (denied at policy).
- **Runbook**: `runbooks/secret-leak-detected.md` §"Residency breach".

## FM-10 — HSM attestation report fails verification

- **Description**: Daily HSM attestation report cannot be verified against vendor's public attestation chain.
- **Trigger**: HSM firmware upgrade lag, attestation key rotation by vendor, supply-chain compromise (T-I-04).
- **Detection**: attestation verifier exit non-zero; `KekAttested` event missing or marked `verification_failed`.
- **Blast radius**: assume HSM compromise until proven otherwise; Sev-1.
- **Mitigation**:
  - Daily verification cron; failure pages immediately.
  - Multi-vendor strategy reduces single-vendor compromise.
- **Recovery**:
  - Investigate with vendor; if compromise confirmed: KEK ceremony in alternate HSM; KEK-of-KEKs rotation; cascade re-wrap all DEKs.
- **RPO/RTO**: RPO 0 (HSM-resident KEK can't be exfiltrated to verify on its own; investigation under Sev-1 process); RTO ≤24h.
- **Runbook**: `runbooks/hsm-key-rotation.md` §"Vendor compromise".

## FM-11 — OpenBao policy mis-author grants over-scope

- **Description**: A PR merges an OpenBao policy that grants a consumer µservice access broader than its intended scope.
- **Trigger**: reviewer miss + LEAN-A12 gap.
- **Detection**: LEAN-A12 `oya-check-openbao-policy-scope` lane (pre-merge); periodic policy-diff audit (post-merge).
- **Blast radius**: depending on scope, could allow cross-µservice or cross-tenant read; Sev-2.
- **Mitigation**:
  - LEAN-A12 lane.
  - Policy tests under `tests/policy/` validate scope per-policy.
  - Reviewer-agent watches policy widenings.
- **Recovery**:
  - Revert policy; audit retroactively for reads under the over-scope window; tenant notification if leaked-read occurred.
- **RPO/RTO**: RPO 0 if caught pre-merge; if post-merge: scope-time window forensic.
- **Runbook**: `runbooks/secret-leak-detected.md` §"Policy mis-author".

## FM-12 — Postgres backend corruption

- **Description**: OpenBao Postgres backend's WAL or table data corruption.
- **Trigger**: storage hardware fault, Patroni misconfig, bad upgrade.
- **Detection**: Postgres error rate; OpenBao read error rate spike; Patroni health probe failures.
- **Blast radius**: pack-wide reads + writes blocked (Sev-1).
- **Mitigation**:
  - Patroni HA with synchronous replication.
  - Daily encrypted backups to pack-pinned object storage.
  - Quarterly restore-from-backup drill.
- **Recovery**:
  - Patroni failover to replica.
  - If both corrupt: restore from latest backup; bounded RPO ≤24h (depending on backup cadence; we use 1h incremental + 24h full).
- **RPO/RTO**: RPO ≤1h; RTO ≤30 min.
- **Runbook**: `runbooks/openbao-restart.md` §"Backend recovery".

## FM-13 — SDK version drift (consumer running stale SDK)

- **Description**: Consumer µservice uses an SDK version that does not honour current revocation push protocol or cache TTL.
- **Trigger**: scheduled-for-distinct-tracked-work upgrade, deprecated SDK still imported, third-party µservice not yet updated.
- **Detection**: `oya_cloud_secrets_sdk_version_count{version}` distribution; deprecation lane in CI.
- **Blast radius**: per-consumer; can break revocation SLA for that consumer's secret reads.
- **Mitigation**:
  - SDK semver + deprecation policy; sunset window ≥6 months.
  - CI lane refuses consumer crates pinning an SDK version > 1 minor behind.
- **Recovery**: upgrade consumer SDK; restart pods.
- **RPO/RTO**: n/a.
- **Runbook**: covered by per-µservice deprecation process.

## FM Catalog Summary

| FM | Sev (worst-case) | Detection latency | RTO | Runbook |
|---|---|---|---|---|
| FM-01 OpenBao quorum loss | Sev-1 | <30s | ≤2 min | openbao-restart |
| FM-02 HSM partition unavailable | Sev-1 (on restart); Sev-2 (running) | <60s | ≤5 min | hsm-key-rotation |
| FM-03 Rotation stuck | Sev-2 | <5 min | ≤30 min | rotation-cascade-recovery |
| FM-04 Audit emission backlog | Sev-2 (Sev-1 if >300s) | <60s | ≤15 min | audit-emission-backlog |
| FM-05 Namespace controller crash | Sev-2 | <10 min | ≤10 min | namespace-controller-restart |
| FM-06 Raw-secret-leak | Sev-1 (always) | varies | ≤5 min revoke | secret-leak-detected |
| FM-07 Cache poisoning | Sev-3 | application-side | seconds | n/a |
| FM-08 Revocation not propagating | Sev-1 (if compromise context); Sev-2 (otherwise) | <5s | ≤30s | secret-leak-detected |
| FM-09 Cross-pack write attempt | Sev-2 (intercepted); Sev-1 if breached | immediate | immediate | secret-leak-detected |
| FM-10 HSM attestation fail | Sev-1 | 24h cycle | ≤24h | hsm-key-rotation |
| FM-11 Policy over-scope | Sev-2 | pre-merge LEAN-A12 | revert immediate | secret-leak-detected |
| FM-12 Postgres corruption | Sev-1 | <60s | ≤30 min | openbao-restart |
| FM-13 SDK version drift | Sev-3 | per-deploy | per-deploy cadence | n/a |

## Drills (cadence)

| Drill | Cadence | Acceptance |
|---|---|---|
| OpenBao Raft chaos | monthly | quorum recovers within 2 min |
| HSM partition failover | quarterly | failover within 5 min |
| Rotation-stuck cascade | monthly | scheduler self-heals |
| Audit emission backlog | quarterly | bridge catches up within 15 min |
| Raw-secret-leak (synthetic) | quarterly | revoke cascade <5s p99 |
| Cross-pack write attempt | quarterly | denied + alerted |
| HSM attestation verify | daily (automated) | passes; deviations paged |
| Postgres backup restore | quarterly | restored cluster reaches healthy within 30 min |

## References

- `microservices/cloud-secrets/threat-model.md`
- `microservices/cloud-secrets/incident-response.md`
- `microservices/cloud-secrets/runbooks/*.md`
- `microservices/cloud-secrets/capacity-model.md`
- Google SRE Workbook ch. 14 ("Managing Load") + ch. 22 ("Addressing Cascading Failures")
