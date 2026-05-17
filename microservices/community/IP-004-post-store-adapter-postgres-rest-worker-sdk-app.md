---
doc_class: ImplementationPlan
template_id: TPL-IP
ip_id: IP-004
microservice: community
phase: PHASE-01-community-substrate
status: Accepted
date: 2026-05-17
owner_team: axis-community
related_adrs: [ADR-0105, ADR-0106, ADR-0126, ADR-0131]
doc_status: published
---

# IP-004 — post-store adapter-postgres + rest + worker + sdk + app crates

## Intent

Land the remaining post-store crate set: `adapter`, `adapter-postgres`, `rest`, `worker`, `sdk`, `app`.

## Scope

- `adapter` — port traits referenced by usecase.
- `adapter-postgres` — Postgres-backed adapter; RLS-aware connection acquisition; uses sqlx; pg_partman partitioning by month.
- `rest` — HTTP gateway implementing `contracts/openapi/community.yaml` for post-store paths.
- `worker` — NATS consumer for search-reindex + audit-bridge tasks.
- `sdk` — Rust client SDK (TypeScript / Python generated separately).
- `app` — composition root; wires kernel → domain → usecase → adapter-postgres → rest + worker.

## Deliverables

- Full crate set landed.
- Integration tests under `tests/`.

## Acceptance

- `cargo test -p oya-community-post-store-* --all-features` green.
- Integration test: `post_create_then_read` against a real Postgres instance.
- Load drill: 1 k creates / s, p99 ≤ 250 ms.
- SLO authored at `slos/post-create.openslo.yaml`.

## Owner

axis-community.
