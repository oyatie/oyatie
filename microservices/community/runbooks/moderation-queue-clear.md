---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: community
runbook_id: moderation-queue-clear
status: Accepted
date: 2026-05-17
owner_team: axis-community + ops-sre
related_artifacts:
  - microservices/community/failure-modes.md (FM-03)
  - microservices/community/threat-model.md (T5.4)
doc_status: published
---

# Runbook: moderation-queue-clear

## When to use

- FM-03 (moderation queue OOM under flag storm)
- T5.4 (coordinated flag campaign)

## Symptoms

- Moderation queue depth > 100 k for a tenant.
- Worker OOM kill rate increases.
- `oya_community_moderation_queue_depth{tenant_id=X}` growing.
- `oya_community_moderation_worker_oom_kill_total` > 0.

## Detection

- Grafana alert `community-moderation-queue-depth-critical`.
- Grafana alert `community-moderation-worker-oom`.

## Triage

1. Identify tenant + target (single post / member / pattern).
2. Determine flag-storm vs. legitimate moderation backlog.
3. Check whether flags are concentrated (coordinated) or dispersed.

## Mitigation

### Coordinated flag campaign

1. Engage foundry-guardrails: extract flagging cluster.
2. Mass-resolve clustered flags as `false-positive`:
   `cargo run -p oya-community-moderation-queue-cli -- mass-resolve --tenant <T> --flag-cluster <id> --verdict false-positive`.
3. Quarantine flagging cluster (`flag` action rate-limit reduced for cluster).
4. Audit-chain seals mass-resolution.

### Legitimate backlog

1. Scale out moderation-queue worker fleet: HPA min from 4 to 16.
2. Engage tenant_admin to surge moderator capacity.
3. Triage flags by priority (PHI / illegal / safety > spam > civility).

### Queue overflow

1. If depth > 500 k: enable cold queue overflow to S3.
2. Worker resumes pulling from S3 cold queue after live queue drains.
3. Verify no flag is lost (audit-chain witness).

## Verification

- Queue depth returns < 10 k.
- Worker OOM rate = 0.
- Moderation action SLO p99 < 200 ms.

## Post-Incident

- If structural: capacity revision; per-tenant queue cap adjustment.
- If coordinated: foundry-guardrails detector tuning.
- Tenant transparency report entry.

## Owner

axis-community (primary) + foundry-guardrails.
