---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: community
runbook_id: spam-flood-throttle
status: Accepted
date: 2026-05-17
owner_team: axis-community + ops-sre
related_artifacts:
  - microservices/community/failure-modes.md (FM-05, FM-09, FM-11, FM-19)
  - microservices/community/incident-response.md
doc_status: published
---

# Runbook: spam-flood-throttle

## When to use

- FM-05 (mass-spam abuse from compromised member)
- FM-09 (Redis hot-feed cache stampede)
- FM-11 (foundry-guardrails bridge backpressure)
- FM-19 (worker pool fair-share starvation)

## Symptoms

- Post create rate z-score > 5 vs. baseline for a tenant.
- Feed render p99 > 800 ms.
- `oya_community_post_create_rate_total{tenant_id=X}` > 5 × tenant baseline.
- foundry-guardrails dead-letter queue depth > 10 k.

## Detection

- Grafana alert `community-post-velocity-anomaly` (z > 5).
- Grafana alert `community-feed-latency-p99-warn` (p99 > 500 ms).
- Grafana alert `community-redis-cpu-warn` (CPU > 80 %).

## Triage

1. Identify tenant_id from alert label.
2. Inspect velocity dashboard: is it one member or many?
3. Check foundry-guardrails classifier signal: stolen-credential vs. distributed bot vs. legitimate event.

## Mitigation

### Single member (compromised credential)

1. Engage tenancy µservice: revoke session for member.
2. Quarantine member: `cargo run -p oya-community-moderation-queue-cli -- quarantine --tenant <T> --member <M>` (sets `banned == true` for 24 h).
3. Verify post create rate returns to baseline.
4. Notify tenant_admin (via tenancy notification channel).

### Distributed bot (multi-member)

1. Engage foundry-guardrails: tighten classifier threshold for tenant (temporary).
2. Apply per-tenant post rate-limit floor: 30 / min (down from 60).
3. Enable account-age gate: members < 24 h cannot post.
4. Tenant_admin notified.

### Legitimate event (e.g., town-hall announcement firestorm)

1. Increase per-tenant capacity headroom: scale gateway + Redis.
2. Pre-warm hot-feed Redis namespace.
3. Coordinate with tenant_admin for moderator surge.

### Redis hot-feed stampede

1. Single-flight feed fill enabled by default; verify it's active (`oya_community_feed_fill_singleflight_active`).
2. Pin trending post to in-memory L1 cache for 5 min.
3. If Redis CPU > 90 % for 5 min: scale out Redis cluster.

### Foundry-guardrails backpressure

1. Inspect bridge lag: `oya_community_guardrails_bridge_lag_seconds`.
2. If classifier model is slow, switch to fast-path rate-limit-only mode (fallback).
3. Manual moderation moves to higher priority in the queue.

## Verification

- Post create velocity p99 within tenant baseline.
- Feed render p99 < 300 ms.
- foundry-guardrails bridge lag < 30 s.

## Post-Incident

- Open ticket for capacity revision if scale-out was triggered.
- If false-positive: tune classifier; document in incident.
- If structural: ADR.

## Owner

axis-community (primary) + ops-sre (secondary).
