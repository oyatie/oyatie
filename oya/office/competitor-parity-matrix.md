---
doc_class: Operational-Doc
shape: Reference
status: Proposed
date: 2026-06-10
owner_team: axis-office
microservice: office
related_adrs:
  - ADR-0131-per-microservice-flat-layout
  - ADR-0510-transitional-substrate-adapters
companion_docs:
  - docs/standards/documentation-rigor.md
---

# oya/office — external reference & competitor parity matrix

External products and codebases used as reference material for the office product
(G012 oyaoffice → oya-office migration). Provenance-only: external project names
appear in reference docs like this one, never in core code or contracts.

| Reference | Type | URL | Use |
|---|---|---|---|
| Euro-Office (GitHub org) | Open-source office suite org | https://github.com/orgs/Euro-Office/repositories | Founder-directed reference (2026-06-10): parity research input for oya/office capabilities; methodology/precedent study only — owned-stack Rust reimplementation per founder doctrine, no code adoption without its own decision record |

## Doctrine

Per the proven-patterns founder doctrine: adopt methodology from references, cite
precedent per decision, reimplement Rust-native. Reference entries here commission
RESEARCH only; any adoption decision requires its own ADR with the precedent cited.
