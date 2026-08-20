---
doc_class: Runbook
title: Rule-store restore (Postgres backup + DR failover)
microservice: foundry-guardrails
severity: "Sev-2 (DR-pair packs) / Sev-1 (single-region) / Sev-2 (backup corruption)"
status: Accepted
owner_team: ops-sre-reliability + axis-foundry-guardrails
date: 2026-05-17
related_artifacts:
  - microservices/intelligence/failure-modes.md (FM-09, FM-10)
  - microservices/intelligence/multi-region.md
  - microservices/intelligence/iac/helm/postgres/values.yaml
doc_status: published
---

# Runbook: Rule-store restore

## Trigger

ONE of:

1. **Postgres rule-store unavailable** (FM-09): primary crash; AZ outage; storage failure.
2. **Backup corruption** (FM-10): daily backup-validation lane detects corruption.
3. **Cross-region restore drill** (BCDR): quarterly scheduled drill per `multi-region.md`.

## Severity

- DR-pair packs (RR promotion): Sev-2.
- Single-region pack (provider-recovery): Sev-1.
- Backup corruption (no live impact yet): Sev-2.

## Pre-checks — primary outage (FM-09)

1. Confirm primary unavailable for ≥ 5 min: `pg_isready -h primary` returns error from two probe locations.
2. Read RR replication lag: `pg_stat_replication.lag` < 1s pre-promotion.
3. Confirm RR is healthy + in steady state.

## Steps — DR-pair RR promotion

| Step | Action | Time |
|---|---|---|
| 1 | Declare Sev-2; open `#inc-<id>` | ≤ 5 min |
| 2 | Verify pre-checks | ≤ 5 min |
| 3 | Promote RR: `bash microservices/intelligence/iac/postgres/failover.sh promote --rr <rr-host> --pack <p>` | ≤ 3 min |
| 4 | Verify new primary accepts writes: synthetic INSERT + SELECT | ≤ 2 min |
| 5 | Update connection pool DNS: TTL drops to 30s; foundry-guardrails pods reconnect | ≤ 1 min |
| 6 | Verify rule-store reads + writes flowing: `foundry_guardrails_rule_store_request_total > 0` | ≤ 2 min |
| 7 | Tenant comms if any Sev-2 user impact | ≤ 30 min |
| 8 | Re-attach old primary as RR once restored | hours |
| 9 | Postmortem within 5 BD | – |

## Steps — single-region pack (provider-recovery; FM-09 Sev-1)

| Step | Action | Time |
|---|---|---|
| 1 | Declare Sev-1; engage ops-sre-reliability + cloud-secrets | ≤ 5 min |
| 2 | Confirm OCI region status (public status page + Oracle support ticket) | ≤ 30 min |
| 3 | foundry-guardrails fail-closed posture engaged: in-pod 5s cache stale → block every invocation per `policy/guardrail-enforcement.md` "fail-closed posture" | automatic |
| 4 | Tenant comms: pack experiencing degradation | ≤ 30 min |
| 5 | If provider recovery > 1h: consider extracting pack to backup region per `multi-region.md` future-ADR | per provider |

## Steps — backup corruption (FM-10)

| Step | Action | Time |
|---|---|---|
| 1 | Quarantine corrupted backup; tag in S3 with `corrupt=true` | ≤ 5 min |
| 2 | Restore from prior validated backup: `bash microservices/intelligence/iac/postgres/restore.sh --backup <prior-id> --pack <p> --target staging-ephemeral` | ≤ 1h |
| 3 | Validate restored data against expected (rule_definitions row count + audit-mutation_log integrity hash) | ≤ 30 min |
| 4 | Investigate corruption cause: pg_dump artifact integrity issue? S3 bit-rot? Provider issue? | days |
| 5 | If pattern emerges (multiple corruptions): consider Reed-Solomon EC at storage tier | weeks |
| 6 | Postmortem within 5 BD | – |

## Steps — BCDR drill (quarterly)

| Step | Action | Time |
|---|---|---|
| 1 | Schedule with tenants (7d notice); maintenance window 30 min | T-7d |
| 2 | Failover via `failover.sh promote` (script-driven; auditable) | ≤ 3 min |
| 3 | Verify pack operations on DR-pair RR | ≤ 5 min |
| 4 | Failback to original primary | ≤ 10 min |
| 5 | Post-drill review: success criteria + lessons | T+1d |

## Verification

After completion:
- New primary serving reads + writes.
- Rule-store SLI green ≥ 30 min.
- Audit-chain seal records the failover / restore event.
- Postmortem (if Sev-1/2).

## Post-incident updates

- Postmortem.
- If FM-09: pattern-of-failure review; consider DR-pair upgrade for single-region pack.
- If FM-10: backup-integrity discipline review; Reed-Solomon consideration.

## References

- `microservices/intelligence/failure-modes.md` FM-09 + FM-10.
- `microservices/intelligence/multi-region.md`.
- `microservices/intelligence/iac/helm/postgres/values.yaml`.
- `microservices/intelligence/iac/postgres/{failover,restore}.sh`.
- PostgreSQL HA + replication — `postgresql.org/docs/16/high-availability.html`.
