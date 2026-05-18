---
doc_class: ImplementationPlan
template_id: TPL-IP
ip_id: IP-010
microservice: community
phase: PHASE-01-community-substrate
status: Accepted
date: 2026-05-17
owner_team: axis-community + foundry-guardrails
related_adrs: [ADR-0105, ADR-0135, ADR-0131]
doc_status: published
---

# IP-010 — foundry-guardrails moderation bridge adapter

## Intent

Ship the bridge adapter that consumes `PostCreated` + `PostEdited` + `VoteCast` events from community and forwards them to foundry-guardrails for spam / abuse / impersonation classification.

## Scope

- Adapter: `oya-community-moderation-queue-adapter-moderation-bridge`.
- Source: NATS JetStream subject `community.<tenant_id>.post.*`.
- Target: foundry-guardrails classifier API.
- Backpressure: dead-letter queue; fallback to rate-limit-only mode on classifier outage.

## Deliverables

- Bridge crate.
- NATS subscription config.
- Fallback policy.
- Per-tenant tunable threshold configuration.

## Acceptance

- Bridge lag p99 ≤ 30 s.
- Dead-letter queue depth alert at > 10 k.
- Fallback mode triggers within 60 s of classifier outage.
- Classifier verdict emits `PostShouldHide` consumed by moderation-queue.

## Owner

axis-community + foundry-guardrails.
