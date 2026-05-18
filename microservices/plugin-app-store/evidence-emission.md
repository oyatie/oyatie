---
doc_class: EvidenceEmission
title: "Evidence emission specification"
microservice: plugin-app-store
status: Accepted
owner_team: axis-ecosystem
date: 2026-05-18
related_adrs: [ADR-0213, ADR-0131]
doc_status: published
---

# Evidence emission specification


## Per-IP evidence

Every IP merge emits to `microservices/<ms>/evidence/multispectrum/<change_id>-<unix_ts>.json` per ADR-0110:

- change_id (ULID)
- ip_id
- microservice
- milestone + phase
- claim_paths (exhaustive list)
- intent (one-line)
- spec_refs (PRD + spec links)
- acceptance_lanes_green
- test_count {unit, integration, e2e}
- coverage_pct
- multispectrum_review_facets (F1..F9 + A1..A7 + M1..M2)
- signature (Ed25519 per ADR-0181)
- executed_at (ISO8601)

## Per-runbook evidence

Every Sev-1/2 incident emits to `microservices/<ms>/evidence/incident-reports/<YYYY-MM-DD>-<slug>.json`:

- incident_id (ULID)
- severity
- triggered_at, resolved_at
- runbook_paths (exhaustive)
- recovery_path_used
- post_mortem_link
- action_items

## Per-pack evidence

Every pack promotion emits to `microservices/<ms>/evidence/pack-promotions/<pack>-<unix_ts>.json`:

- pack_id
- promotion_lane
- canonical_base_neutrality_check (must be true)
- regulatory_audit_links

## Daily integrity evidence

Daily audit-chain integrity verification emits to `microservices/<ms>/evidence/chain-integrity/<YYYY-MM-DD>.json`:

- date
- chain_head_hash
- chain_length
- verification_result {pass|fail}
- discrepancies (if any)

## Verification

`cargo run -p oya-dev-cli -- gate validate evidence-emission --microservice <ms>` exits 0 only if every evidence schema is present + populated.

