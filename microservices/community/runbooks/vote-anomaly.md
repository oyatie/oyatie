---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: community
runbook_id: vote-anomaly
status: Accepted
date: 2026-05-17
owner_team: axis-community + ops-sre
related_artifacts:
  - microservices/community/failure-modes.md (FM-02)
  - microservices/community/threat-model.md (T2.2, T5.2)
doc_status: published
---

# Runbook: vote-anomaly

## When to use

- FM-02 (vote race / double-count)
- T2.2 (vote manipulation by bot farm)
- T5.2 (vote-storm against a target post)

## Symptoms

- Vote rate z-score > 5 for a single post.
- Vote tally divergence between Redis and Postgres > 0.1 %.
- `oya_community_vote_cast_total` for a (post_id) growing > 100 / min.
- Coordinated cohort detector emits alert.

## Detection

- Grafana alert `community-vote-velocity-anomaly`.
- Grafana alert `community-vote-tally-divergence`.
- foundry-guardrails coordinated-cohort detector alert.

## Triage

1. Identify target post + tenant.
2. Check whether it's:
   - Single bot farm (one cluster, same IP / UA family)
   - Distributed brigading (many IPs, geographically diverse)
   - Legitimate viral event (trending KB article release)
3. Check vote tally divergence: Redis vs. Postgres counter.

## Mitigation

### Bot farm

1. Engage foundry-guardrails: extract member cluster.
2. Quarantine cluster members (`banned == true` for 24 h).
3. Reverse votes from quarantined cluster:
   `cargo run -p oya-community-voting-engine-cli -- reverse --tenant <T> --since <ts> --members <list>`.
4. Recompute tally.
5. Audit-chain emits `VoteReversalApplied` per (post, member).

### Distributed brigading

1. Tighten coordinated-cohort detector threshold for tenant temporarily.
2. Per-post temporary vote freeze on the target post.
3. Tenant_admin notification with member list.

### Vote tally divergence

1. Inspect `oya_community_vote_tally_divergence_total`.
2. Pause vote writes for affected post.
3. Reconcile: rebuild tally from Postgres source of truth via worker job.
4. Resume vote writes.
5. Audit-chain emits `VoteTallyReconciled`.

### Legitimate viral event

1. Verify content + author legitimacy with tenant_admin.
2. Whitelist post temporarily (suppress velocity alert).
3. No reversal.

## Verification

- Vote velocity returns to baseline.
- Tally divergence < 0.01 %.
- Audit-chain seals all reversals.

## Post-Incident

- If structural: idempotency key audit; possibly tighten Redis Lua script.
- Coordinate with foundry-guardrails on detector tuning.
- Per-tenant transparency report entry.

## Owner

axis-community (primary) + foundry-guardrails (classifier).
