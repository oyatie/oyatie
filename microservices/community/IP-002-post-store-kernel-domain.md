---
doc_class: ImplementationPlan
template_id: TPL-IP
ip_id: IP-002
microservice: community
phase: PHASE-01-community-substrate
status: Accepted
date: 2026-05-17
owner_team: axis-community
related_adrs: [ADR-0105, ADR-0106, ADR-0126, ADR-0131]
doc_status: published
---

# IP-002 — post-store kernel + domain crates

## Intent

Ship `oya-community-post-store-kernel` (types + invariants) and `oya-community-post-store-domain` (aggregate + business rules) per ADR-0105 13-layer enum.

## Scope

- Types: `Post`, `Author`, `Mention`, `Revision`, `SpaceRef`, `PostKind`, `ModerationState`.
- Invariants: revision append-only; author_id from JWT claim; body length ≤ 100k.
- Domain aggregate: `PostAggregate` with `author`, `edit`, `delete`, `mention`, `tag`, `link_ontology` methods.
- No I/O; no async; pure compute.

## Deliverables

- Crate `oya-community-post-store-kernel` (this IP)
- Crate `oya-community-post-store-domain` (this IP)
- Catalog entries in `catalog/`

## Acceptance

- `cargo test -p oya-community-post-store-kernel` green.
- `cargo test -p oya-community-post-store-domain` green.
- 100 % test coverage on domain invariants.
- Doc coverage gate (lean-a5) green.

## Owner

axis-community.
