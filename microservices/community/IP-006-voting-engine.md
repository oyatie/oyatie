---
doc_class: ImplementationPlan
template_id: TPL-IP
ip_id: IP-006
microservice: community
phase: PHASE-01-community-substrate
status: Accepted
date: 2026-05-17
owner_team: axis-community
related_adrs: [ADR-0105, ADR-0135, ADR-0131]
doc_status: published
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-006 — voting-engine

## Intent

Ship the voting BC with conflict-free counter (Redis Lua) and idempotent vote cast.

## Scope

- Types: `Vote`, `Tally`, `Acceptance`, `VoteDirection`.
- Storage: Redis (live counter) + Postgres (source of truth + audit).
- Operations: `cast_vote(post_id, direction, idempotency_key)`, `read_tally(post_id)`, `accept_answer(question_id, reply_id)`.
- Redis Lua script for atomic SET NX + INCRBY + audit append.
- Async Postgres flush via worker.

## Deliverables

- Crate set: kernel + domain + usecase + api + adapter + adapter-postgres + worker + sdk.
- Redis Lua script in `src/lua/vote_cast.lua`.
- Reconciliation worker for Redis vs. Postgres divergence detection.

## Acceptance

- Vote cast p99 ≤ 100 ms.
- Idempotency verified by integration test.
- Divergence detector runs hourly; alerts at > 0.1 %.
- SLO authored.

## Owner

axis-community.
