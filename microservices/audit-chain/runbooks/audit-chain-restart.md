---
doc_class: Runbook
title: Audit-chain component restart procedures
microservice: audit-chain
severity: varies by FM
status: Accepted
owner_team: ops-sre-reliability + axis-audit-chain
date: 2026-05-17
related_artifacts:
  - microservices/audit-chain/failure-modes.md (FM-01, FM-05, FM-06, FM-07, FM-08)
  - microservices/audit-chain/policy/seal-integrity.md
  - microservices/audit-chain/multi-region.md
doc_status: published
---

# Runbook: Audit-chain component restart procedures

## Purpose

Component-level restart + recovery procedures for emission-rest, sealing-worker, verification-rest, query-rest, retention-cascade-worker, Postgres, S3 WORM, HSM. Cross-references `failure-modes.md` for trigger conditions.

## emission-rest Recovery

### Pre-checks
1. Pod state: `kubectl -n audit-chain get pods -l app=emission-rest`.
2. Logs: `kubectl -n audit-chain logs -l app=emission-rest --tail=200`.
3. Postgres reachability: `kubectl exec <pod> -- psql -c 'SELECT 1'`.
4. OpenBao token renewal: logs show `openbao_token_renewed_total{}` rate > 0.

### Recovery Path A — Crashloop single pod
| Step | Action |
|---|---|
| 1 | Identify cause from logs; common: panic on malformed payload (size > 1MB caller passed); rate-limit bug; OpenBao auth failure |
| 2 | If recent deploy: roll back via ArgoCD or `kubectl rollout undo` |
| 3 | If OpenBao: rotate SA token; reschedule pod |
| 4 | Verify: pod stable for ≥ 5 min |

### Recovery Path B — Overload (FM-08)
| Step | Action |
|---|---|
| 1 | HPA should auto-scale; if not, manual scale `kubectl scale deployment emission-rest-<pack> --replicas=<higher>` |
| 2 | Identify offending source µservice: query `oya_audit_chain_rate_limit_429_total` by `source_microservice` label |
| 3 | Engage offending workload owner |
| 4 | Tenant rate limit enforcement: ensure per-source caps are configured |

## sealing-worker Recovery (FM-07)

### Pre-checks
1. Pod state: `kubectl -n audit-chain get pods -l app=sealing-worker-<pack>`.
2. Per-shard leader-lease state: `kubectl get lease sealing-worker-<pack>-<shard>-leader -o yaml`.
3. HSM partition reachability + OpenBao session validity.
4. Postgres WAL reachability.

### Recovery Path A — Single replica crashloop
| Step | Action |
|---|---|
| 1 | HA leader-election failover to standby replica within ≤ 5 min |
| 2 | Diagnose root cause from crashed pod's logs |
| 3 | If Merkle-build bug on malformed event: file Issue; pin known-good version; redeploy |

### Recovery Path B — Both replicas down per shard
| Step | Action |
|---|---|
| 1 | Declare Sev-2 (Sev-1 if persistent > 1h or affects regulator-bound SLAs); engage axis-audit-chain on-call + Cryptography SME |
| 2 | Verify HSM partition health (if down, see HSM Recovery below) |
| 3 | Verify Postgres health |
| 4 | If shared dependency is the issue, fix root cause first |
| 5 | If worker code: emergency hotfix PR; deploy via Helm; emergency-merge sign-off |
| 6 | Resume worker; verify catch-up |

### Recovery Path C — Leader-election storm
| Step | Action |
|---|---|
| 1 | Symptom: replicas constantly handing off; sealing paused |
| 2 | Verify lease: `kubectl get lease sealing-worker-<pack>-<shard>-leader -o yaml` |
| 3 | Increase `leaseDurationSeconds` to 30s; redeploy |

## verification-rest Recovery

Stateless; recovery via HPA. If down across all replicas:
1. Investigate KeyResolver cache health (if KeyResolver Postgres lookup fails, all verifications fail).
2. Restart pods.
3. Verify with sample request.

## query-rest Recovery

Stateless; recovery via HPA. If down:
1. Check Postgres index health.
2. Restart.

## retention-cascade-worker Recovery

See `runbooks/retention-cascade.md` for procedural depth.

## Postgres Recovery (FM-05)

### Pre-checks
1. Primary state: `kubectl -n audit-chain get pods -l role=primary`.
2. Replica lag: `pg_replication_lag_seconds`.
3. Connection role audit: `pg_role_audit` cronjob.

### Procedure
| Step | Action |
|---|---|
| 1 | If primary down: promote replica via Postgres Operator |
| 2 | Verify role-grant invariants (T-E-01): `audit_emitter` INSERT-only; `audit_sealer` SELECT + INSERT on SealRecord |
| 3 | emission-rest auto-reconnects to new primary; verify backlog drained from local-WAL-on-disk fallback |
| 4 | sealing-worker re-reads WAL; verifies S3 raw-blob as source-of-truth per `policy/seal-integrity.md` §"T-T-01 defence-in-depth" |

## S3 WORM Recovery (FM-06)

### Pre-checks
1. OCI Object Storage status.
2. Bucket policy: Object Lock Compliance still enforced.

### Procedure
| Step | Action |
|---|---|
| 1 | If bucket unavailable: queue raw-blob writes in sealing-worker local buffer |
| 2 | DR-pair pack: failover S3 to DR-pair bucket (via Global Traffic Manager DNS) |
| 3 | Single-region pack: await Oracle recovery |
| 4 | Once restored: drain queued writes; verify each blob's SHA matches Postgres index |

## HSM Recovery (FM-01)

See `runbooks/hsm-key-rotation.md` for full procedure. Brief summary:
1. HSM unreachable: emission continues; sealing degraded.
2. DR-pair pack: failover to DR-pair partition (intra-pack OCI HSM partition replication).
3. Single-region pack: await Oracle.
4. Once restored: sealing-worker batch-seals accumulated events.

## On-call Handoff

Standard handoff per oyatie operating practice. Audit-chain-specific:
- On-call must understand the 3 Sev-1 chain-integrity FMs (FM-02, FM-03, FM-04) before solo rotation.
- Cryptography SME engaged via paged handoff for any chain-integrity event.

## Verification (post-restart)

- Component health probe `200 OK`.
- Pod stable ≥ 5 min.
- No backlog accumulating.
- Self-SLO returns to green.
- Sample emit → verify round-trip succeeds.

## References

- `microservices/audit-chain/failure-modes.md`.
- `microservices/audit-chain/policy/seal-integrity.md`.
- `microservices/audit-chain/multi-region.md`.
- `microservices/audit-chain/runbooks/hsm-key-rotation.md`.
- `microservices/audit-chain/runbooks/retention-cascade.md`.
- Kubernetes lease docs.
- OCI Cloud-HSM ops docs.
