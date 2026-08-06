---
doc_class: Runbook
title: Mention storm throttle
microservice: messenger
severity: "Sev-3 (degraded) / Sev-2 (persistent)"
status: Accepted
owner_team: axis-messenger
date: 2026-05-17
related_artifacts:
  - microservices/messenger/failure-modes.md (FM-08)
  - microservices/messenger/capacity-model.md
doc_status: published
---

# Runbook: Mention storm throttle (FM-08)

## Trigger

- `messenger_mention_fanout_queue_depth` > 100k.
- Per-message mention recipient count > 500 (over-cap attempted).
- One channel produces > 1k mentions/min sustained.

## Severity

Sev-3 default; escalate to Sev-2 if delivery lag > 5 min sustained.

## Immediate Mitigation (≤ 15 min)

| Step | Action | Time |
|---|---|---|
| 1 | Inspect queue depth breakdown: `messenger_mention_fanout_queue_depth` by `tenant_id`, `channel_id` | ≤ 2 min |
| 2 | Identify hot channel: top-emitter | ≤ 3 min |
| 3 | If single tenant or channel: tighten per-message mention cap (default 50 → temporary 10) | ≤ 5 min |
| 4 | Engage tenant security-admin (potential bot abuse) | ≤ 5 min |
| 5 | Scale mention-router workers (HPA or manual) | ≤ 5 min |
| 6 | If queue backlog > 30 min budget: drop low-priority repeat-mention dedup | ≤ 5 min |
| 7 | Notification rate-limit: per-recipient mention-notification throttle to 10/min during storm | ≤ 5 min |

## Diagnosis

| Hypothesis | Signal | Action |
|---|---|---|
| Bot script flooding | Author = service account; mass-identical messages | engage tenant security-admin; revoke bot token |
| Legitimate broadcast (org-wide announcement) | Single message; channel size = entire tenant | accept as legitimate; absorb cost |
| Mention parser bug | Mention count >> actual @mentions in body | code review; rollback recent mention-router release |
| Ontology lookup latency cascade | mention-resolve p99 spike | engage axis-ontology |

## Recovery Verification

- `messenger_mention_fanout_queue_depth` < 5k for ≥ 30 min.
- `messenger_mention_resolution_p99_seconds` ≤ 0.25.
- Per-channel emission rate within normal envelope.

## Postmortem

- If pattern (bot abuse): introduce tenant-scoped bot rate-limit policy.
- If pattern (legitimate broadcast): communicate to tenant about scale-pack upgrade.
- Review per-tenant mention caps in `capacity-model.md`.

## References

- `microservices/messenger/failure-modes.md` FM-08.
- `microservices/messenger/capacity-model.md` §"Mention-Router Sizing".
- ontology µservice integration docs.
