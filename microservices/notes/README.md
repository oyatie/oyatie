---
doc_class: Reference
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0245]
companion_docs:
  - microservices/notes/PRD.md
  - microservices/notes/ARCHITECTURE.md
  - microservices/notes/manifest.json
inbound_citations:
  - docs/README.md
---

# Notes µservice — README

## What this µservice does

Notes is oyatie's personal + work notes product. Hyperscaler precedent: Notion + Obsidian + Apple Notes + Bear + Roam Research + Logseq + Craft + Reflect + Mem.ai. Personal workspaces default to E2E encryption (per ADR-NOTES-0001); work workspaces support server-side search + intelligence with tenant-admin gating.

## Quick links

- Product requirements: `PRD.md`
- Architecture: `ARCHITECTURE.md`
- Threat model: `threat-model.md`
- DPIA: `dpia.md`
- Compliance: `compliance.md`
- Capacity / cost / failure modes / multi-region / incident-response / backfill: `*.md`
- Competitor parity: `competitor-parity-matrix.md`
- SDK plan: `sdk-plan.md`
- Contracts: `contracts/{openapi,asyncapi,proto}/`
- Cedar fragments: `policy/*.cedar`
- Runbooks: `runbooks/*.md`
- IPs: `IP-*.md`
- Dashboards: `dashboards/*`
- SLOs: `slos/*.openslo.yaml`
- Catalog: `catalog/*.yaml`
- IaC: `iac/**`

## How to consume

- Native + web clients: sync surface over HTTP/3 + QUIC.
- Backups + export: portable Markdown + JSON archive per ADR-0276.
- Collab-edit: MLS Group + Loro CRDT.

## Status

Product, ga. Adopts the ADR-0330 `tenant_class` model: `demo_trial` tenants use capped OCI Always Free profiles where applicable, and `paid` tenants use composable `billing_components`. HIPAA + PCI pack overlays are `compliance_pack` gated.
