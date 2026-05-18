---
doc_class: ImplementationPlan
template_id: TPL-IP
ip_id: IP-003
microservice: community
phase: PHASE-01-community-substrate
status: Accepted
date: 2026-05-17
owner_team: axis-community
related_adrs: [ADR-0105, ADR-0106, ADR-0135, ADR-0131]
doc_status: published
---

# IP-003 — post-store usecase + api crates

## Intent

Ship `oya-community-post-store-usecase` (commands + queries) and `oya-community-post-store-api` (protocol-neutral typed contracts) per ADR-0105 13-layer + ADR-0106 rename.

## Scope

- Commands: `CreatePost`, `EditPost`, `DeletePost`, `LinkOntology`, `TagPost`.
- Queries: `ReadPost`, `ListPosts`, `ListPostsByTag`, `ListPostsByAuthor`.
- Use-case orchestrates domain + adapter ports.
- API: stable typed surfaces consumed by `-rest`, `-sdk`, and any future protocol adapter.

## Deliverables

- Crate `oya-community-post-store-usecase`
- Crate `oya-community-post-store-api`
- Catalog entries in `catalog/`
- Cedar action enumeration matched to use-case methods

## Acceptance

- `cargo test -p oya-community-post-store-usecase --features ports-mock` green.
- API contract surface frozen at v0.1.
- Cedar fragment coverage gate green.

## Owner

axis-community.
