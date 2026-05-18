---
doc_class: Runbook
title: Mention storm throttle
microservice: social
severity: "Sev-3 (degraded) / Sev-2 (persistent)"
status: Accepted
owner_team: axis-social
date: 2026-05-17
related_artifacts:
  - microservices/social/failure-modes.md (FM-08)
  - microservices/social/capacity-model.md
doc_status: published
---

# Runbook: Mention storm throttle (FM-08)

## Trigger

- `social_mention_fanout_queue_depth` > 100k.
- Per-post mention recipient count > 500 (over-cap attempted).
- One account produces > 1k mentions/min sustained.

## Severity

Sev-3 default; escalate to Sev-2 if delivery lag > 5 min sustained.

## Immediate Mitigation (≤ 15 min)

| Step | Action | Time |
|---|---|---|
| 1 | Inspect queue depth breakdown: `social_mention_fanout_queue_depth` by `tenant_id`, `account_ref` | ≤ 2 min |
| 2 | Identify hot account: top-emitter | ≤ 3 min |
| 3 | If single tenant or account: tighten per-post mention cap (default 50 → temporary 10) | ≤ 5 min |
| 4 | Engage tenant security-admin (potential bot abuse) | ≤ 5 min |
| 5 | Scale mention-router workers (HPA or manual) | ≤ 5 min |
| 6 | If queue backlog > 30 min budget: drop low-priority repeat-mention dedup | ≤ 5 min |
| 7 | Notification rate-limit: per-recipient mention-notification throttle to 10/min during storm | ≤ 5 min |

## Diagnosis

| Hypothesis | Signal | Action |
|---|---|---|
| Bot script flooding | Author = service account; mass-identical posts | engage tenant security-admin; revoke bot token |
| Legitimate broadcast (viral mention thread) | Single post; thread chain self-amplifies | accept as legitimate; absorb cost; capacity review |
| Mention parser bug | Mention count >> actual @mentions in body | code review; rollback recent mention-router release |
| Ontology lookup latency cascade | mention-resolve p99 spike | engage axis-ontology |
| Sybil-amplification attack (related to trending) | foundry-guardrails signal + clustering | engage ops-security; per-IP block; per-tenant attack-mode toggle |

## Recovery Verification

- `social_mention_fanout_queue_depth` < 5k for ≥ 30 min.
- `social_mention_resolution_p99_seconds` ≤ 0.25.
- Per-account emission rate within normal envelope.

## Postmortem

- If pattern (bot abuse): introduce tenant-scoped bot rate-limit policy.
- If pattern (legitimate viral mention): communicate to tenant about scale-pack upgrade.
- Review per-account mention caps in `capacity-model.md`.
- If sybil-amplification confirmed: coordinate with `trending-topic-poisoning.md` runbook.

## References

- `microservices/social/failure-modes.md` FM-08.
- `microservices/social/capacity-model.md` §"Notification Fanout Sizing".
- ontology µservice integration docs.
