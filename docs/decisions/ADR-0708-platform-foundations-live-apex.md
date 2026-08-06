---
id: ADR-708
title: "Live platform foundations: cells, residency, workflow, plugins, search"
status: Accepted
planning_impact: true
deciders: founder
date: 2026-08-06
door: two-way
owner: council-architecture
supersedes: [ADR-49, ADR-158, ADR-240, ADR-251, ADR-253]
superseded_by: []
amends: []
amended_by: []
depends_on: [ADR-515, ADR-363, ADR-562]
related: []
milestone: W0
---
# ADR-708: Live platform foundations: cells, residency, workflow, plugins, search

## Status

**Accepted** — live consolidated source-of-truth entry for topic `platform_foundations` (E5 2026-08-06).

## Context

Oyatie ADR corpus cleanup: agents must not treat every historical Accepted file as equal live law.
This apex consolidates **5** Accepted ADRs in the `platform_foundations` topic. Member files are
**Superseded** by this apex and then archived; full text remains in git history.

Live resolution: prefer this apex; follow `supersedes` for provenance.

## Decision

1. **This ADR is the live reading entry** for topic `platform_foundations` under the end-state ADR policy.
2. **Member ADRs listed in `supersedes`** are historical; normative gist is preserved below.
3. **Contradictions** among members are resolved by later higher-number members and by
   ADR-0515 / ADR-0363 / ADR-0562 / ADR-0615 / ADR-0635 / ADR-0637–0639 when applicable.
4. **Activation-sensitive** items (warm CAS, RE workers) remain fail-closed until explicit go-gate.

## Preserved member gists

- **ADR-49** (ADR-0049-cross-region-replication-and-residency): We adopt **per-pack default residency class** (`strict_kr` / `kr_with_us_failover` / `global` at GA; per-pack additions later); **cross-region replication opt-in per residency class per Data Use Boundary** (per ADR-0008); **tenant residency immutable post-create** (residency change requires re-create of tenant + DSR cascade on the old cell); per-pa
- **ADR-158** (ADR-0158-multi-region-active-active): Every oyatie µservice declares one of three multi-region dispositions in its `manifest.json` under `multi_region_disposition`: - **`active_active`** — multiple write-able regions; data converges via the µservice's chosen consistency model (CRDT per ADR-0142, or quorum-write per Spanner-class semantics, or append-only-merge for audit-chain). - **`ac
- **ADR-240** (ADR-0240-sovereign-cloud-per-regional-pack): ### D-1. Per-pack overlay declaration Each regional pack declares a `sovereign_cloud_overlay.yaml` at `regional-packs/<pack-id>/sovereign-cloud-overlay.yaml` with the following shape: ```yaml pack_id: kr primary_provider: id: naver-cloud regions: [kr-seoul, kr-busan] certifications: [CSAP-1.0, K-ISMS-P] contract_id: NAVER-CLOUD-2026-001 secondary_p
- **ADR-251** (ADR-0251-compliance-pack-cell-certification-levels): ### D-1. Compliance Pack schema (canonical) A Compliance Pack is a versioned, signed bundle described by the following JSON Schema (canonical at `/specs/compliance-pack-schema.json`): ```json { "$schema": "https://json-schema.org/draft/2020-12/schema", "$id": "https://specs.oyatie/compliance-pack-schema.json", "title": "Compliance Pack", "type": "o
- **ADR-253** (ADR-0253-network-topology-edge-service-mesh): ### D-1. Apex DNS — Anycast + GeoDNS, externally hosted first, self-hosted Year 3+ The apex DNS for oyatie operates as **Anycast + GeoDNS at the planet's edge**. **Year 1-2 (externally hosted):** - **Primary registrar + DNS provider:** Cloudflare DNS (Anycast across 300+ POPs) + AWS Route 53 (Anycast across 100+ POPs) as **dual authoritative** for 

## Consequences

- Agent default read path: `docs/decisions/ADR-0xxx` apex files + this topic.
- Citations to member numbers remain valid via `docs/decisions/_disposition/adr-redirect.v1.json`.
- Further body merge refinements may land as amendments to this apex only.

### ADR-158 residual

**ADR-0158-multi-region-active-active** — Every oyatie µservice declares one of three multi-region dispositions in its `manifest.json` under `multi_region_disposition`: - **`active_active`** — multiple write-able regions; data converges via the µservice's chosen consistency model (CRDT per ADR-0142, or quorum-write per Spanner-class semantics, or append-only-merge for audit-chain). - **`active_passive`** — one primary region; one or more

### ADR-240 residual

**ADR-0240-sovereign-cloud-per-regional-pack** — ### D-1. Per-pack overlay declaration Each regional pack declares a `sovereign_cloud_overlay.yaml` at `regional-packs/<pack-id>/sovereign-cloud-overlay.yaml` with the following shape: ```yaml pack_id: kr primary_provider: id: naver-cloud regions: [kr-seoul, kr-busan] certifications: [CSAP-1.0, K-ISMS-P] contract_id: NAVER-CLOUD-2026-001 secondary_provider: id: kt-cloud regions: [kr-seoul-dr] certi

### ADR-251 residual

**ADR-0251-compliance-pack-cell-certification-levels** — ### D-1. Compliance Pack schema (canonical) A Compliance Pack is a versioned, signed bundle described by the following JSON Schema (canonical at `/specs/compliance-pack-schema.json`): ```json { "$schema": "https://json-schema.org/draft/2020-12/schema", "$id": "https://specs.oyatie/compliance-pack-schema.json", "title": "Compliance Pack", "type": "object", "required": [ "pack_id", "version", "regul

### ADR-253 residual

**ADR-0253-network-topology-edge-service-mesh** — ### D-1. Apex DNS — Anycast + GeoDNS, externally hosted first, self-hosted Year 3+ The apex DNS for oyatie operates as **Anycast + GeoDNS at the planet's edge**. **Year 1-2 (externally hosted):** - **Primary registrar + DNS provider:** Cloudflare DNS (Anycast across 300+ POPs) + AWS Route 53 (Anycast across 100+ POPs) as **dual authoritative** for redundancy. Per ADR-0240 sovereign-cloud-per- regi

### ADR-49 residual

**ADR-0049-cross-region-replication-and-residency** — We adopt **per-pack default residency class** (`strict_kr` / `kr_with_us_failover` / `global` at GA; per-pack additions later); **cross-region replication opt-in per residency class per Data Use Boundary** (per ADR-0008); **tenant residency immutable post-create** (residency change requires re-create of tenant + DSR cascade on the old cell); per-pack regulator-binding per region; cross-region tran
