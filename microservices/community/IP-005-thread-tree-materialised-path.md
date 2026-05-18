---
doc_class: ImplementationPlan
template_id: TPL-IP
ip_id: IP-005
microservice: community
phase: PHASE-01-community-substrate
status: Accepted
date: 2026-05-17
owner_team: axis-community
related_adrs: [ADR-0105, ADR-0135, ADR-0131]
doc_status: published
---

# IP-005 — thread-tree (materialised path)

## Intent

Ship the threaded-reply BC with materialised-path indexing for sub-second deep-tree traversal.

## Scope

- Types: `Thread`, `Node`, `Path` (LTREE), `Depth`.
- Storage: Postgres LTREE; index on path.
- Operations: `list_replies(post_id, depth)`, `post_reply(parent_node_id, body)`, `read_thread(thread_id)`.
- Crate set: kernel + domain + usecase + api + adapter + adapter-postgres + sdk (no rest; consumed through post-store rest).

## Deliverables

- Crate set landed.
- LTREE schema migration.

## Acceptance

- 10⁴-node tree render p99 ≤ 350 ms.
- LTREE index used for all path queries.
- `cargo test -p oya-community-thread-tree-*` green.

## Owner

axis-community.
