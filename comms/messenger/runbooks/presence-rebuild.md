---
doc_class: Runbook
title: Presence rebuild
microservice: messenger
severity: "Sev-2 (presence stale globally) / Sev-3 (single shard)"
status: Accepted
owner_team: ops-sre-reliability + axis-messenger
date: 2026-05-17
related_artifacts:
  - microservices/messenger/failure-modes.md (FM-03, FM-11)
  - microservices/messenger/capacity-model.md
doc_status: published
---

# Runbook: Presence rebuild (FM-03, FM-11)

## Trigger

- `messenger_presence_inconsistency_total` > 0 sustained.
- Presence stale globally for > 5 min.
- Valkey cluster split-brain or AOF corruption.

## Severity

- Single Valkey shard issue: Sev-3.
- Multiple shards or global presence outage: Sev-2.
- Combined with message-send breakage: escalate to Sev-1.

## Immediate Mitigation (≤ 15 min)

| Step | Action | Time |
|---|---|---|
| 1 | Verify Valkey cluster health: `valkey-cli --cluster check valkey-primary.messenger.svc:6379` | ≤ 2 min |
| 2 | If shard down: failover replica to primary; promote replica | ≤ 5 min |
| 3 | Flush stale presence keys for affected shard: `KEYS pattern → DEL` (rebuilds from active sessions) | ≤ 5 min |
| 4 | Trigger gateway re-emission: each gateway pod walks its active connection table and re-emits presence | ≤ 5 min |
| 5 | Verify presence consistency: random user query returns latest state | ≤ 2 min |

## Read-Receipt Path (FM-11)

If read-receipt coalescer storms (separate Valkey usage):

| Step | Action |
|---|---|
| 1 | Widen coalesce window from 250ms to 1s via runtime config flag |
| 2 | Scale read-receipt-tracker-worker replicas (default min 4 → 8) |
| 3 | Drain receipt-worker queue; re-emit from Postgres last_read_cursor |
| 4 | Narrow coalesce window back to 250ms over next 30 min once stabilised |

## Diagnosis

| Hypothesis | Signal | Investigation |
|---|---|---|
| AOF corruption | Valkey startup logs | restore from RDB snapshot |
| Split-brain | conflicting cluster topology | enforce single-primary via sentinel failover |
| Eviction storm | `evicted_keys` growth | check memory pressure; scale shards |
| Network partition | inter-AZ latency spike | engage cloud-k8s + cloud-iac |

## Recovery Verification

- `messenger_presence_propagation_p99_seconds` ≤ 1.0.
- `messenger_presence_inconsistency_total` rate = 0 for ≥ 15 min.
- `messenger_read_receipt_coalesce_window_breach_total` rate = 0.

## Postmortem

- If recurring (≥ 2 in 90d): redesign presence resilience.
- If Valkey cluster capacity insufficient: revisit sizing.

## References

- `microservices/messenger/failure-modes.md` FM-03, FM-11.
- `microservices/messenger/capacity-model.md` §"Valkey Presence".
- Valkey cluster docs.
