---
doc_class: ImplementationPlan
template_id: TPL-IP
ip_id: IP-007
microservice: community
phase: PHASE-01-community-substrate
status: Accepted
date: 2026-05-17
owner_team: axis-community
related_adrs: [ADR-0028, ADR-0105, ADR-0126, ADR-0131]
doc_status: published
---

# IP-007 — moderation-queue

## Intent

Ship the moderation BC with append-only action log, audit-chain seal per action, and two-eyes enforcement for destructive verbs.

## Scope

- Types: `Flag`, `Action`, `QueueItem`, `ModeratorVerdict`, `FlagReason`, `ModerationVerb`.
- Storage: Postgres with append-only trigger on `moderation_actions`.
- Operations: `raise_flag`, `resolve_flag`, `apply_action`, `list_queue`, `escalate_flag`.
- Audit-chain seal per action (Ed25519 signed by moderator JWT-bound key).
- Two-eyes enforcement: `delete_post > 100/day per mod` requires approver.

## Deliverables

- Crate set: kernel + domain + usecase + api + adapter + adapter-postgres + adapter-moderation-bridge + worker + sdk.
- Postgres trigger script preventing UPDATE/DELETE on `moderation_actions`.
- Audit-chain integration hook.

## Acceptance

- Moderation action p99 ≤ 200 ms.
- Append-only invariant verified by attempted UPDATE/DELETE (rejected).
- Two-eyes verified by integration test.
- Audit-chain seal latency p99 ≤ 1 s.

## Owner

axis-community + ops-security.
