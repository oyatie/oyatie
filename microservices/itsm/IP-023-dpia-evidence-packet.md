---
doc_class: IP
ip_id: IP-023-dpia-evidence-packet
microservice: itsm
status: rewritten-wave-15-ip-substance
date: 2026-05-21
owner_team: axis-itsm + privacy
counterparts: [ServiceNow ITSM, Jira Service Management, Freshservice]
source_artifacts:
  - microservices/itsm/manifest.json
  - microservices/itsm/policy/data-residency.md
  - microservices/itsm/src/domain/mod.rs
  - microservices/itsm/contracts/openapi-v1.yaml
---

# IP-023 ITSM DPIA Evidence Packet

## A. Problem
ITSM processes requester identity, incident descriptions, asset ownership, PHI/PII under some packs, and postmortem evidence. A privacy reviewer needs a concrete DPIA packet, not a stamped assertion that "privacy is covered."

This IP defines the evidence packet for GDPR, KR-PIPA, HIPAA, and similar pack reviews.

## B. Approach
Generate DPIA evidence from existing artifacts:

| DPIA section | ITSM evidence |
|---|---|
| processing purpose | REST `purpose`, usecase name, ticket/status action |
| data categories | `DataClass` and contract payload fields |
| recipients | workflow-engine, audit-chain, observability, marketplace |
| residency | `policy/data-residency.md` and cell layout |
| retention | pack overlay and postmortem/audit retention |
| safeguards | Cedar, tenant id, audit-chain, OpenBao references |

## C. Deliverables
- DPIA packet template populated with real ITSM paths and data classes.
- Mapping from OpenAPI fields to `DataClass`.
- Evidence links to Cedar policy, residency policy, dashboards, and tests.
- Privacy-specific acceptance tests for redaction and cross-region denial.
- Export bundle suitable for auditor review.

## D. Implementation
1. Inventory personal and quasi-personal data in `contracts/openapi-v1.yaml` and domain types.
2. Map each field to purpose, data class, retention, and recipient µservice.
3. Link residency rules from IP-015 and emergency behavior from IP-013.
4. Document processor/subprocessor boundaries for ServiceNow/Jira/Freshservice imports.
5. Add tests proving public status updates redact requester and internal CI details.
6. Add tests proving cross-region DPIA export obeys pack rules.
7. Emit audit evidence when a DPIA packet is generated or exported.
8. Record follow-up gaps where source artifacts are missing instead of inventing them.

## E. Acceptance
- DPIA packet names real ITSM contract fields and domain data classes.
- GDPR/KR-PIPA/HIPAA pack behavior is explicit.
- Export bundle includes evidence hashes and source artifact paths.
- No counterpart import id is treated as tenant authority.

## F. Evidence
- `manifest.json` lists privacy-sensitive packs.
- `policy/data-residency.md` is the residency policy surface.
- `src/domain/mod.rs` defines `DataClass`.
- `contracts/openapi-v1.yaml` defines `ActionRequest` fields.

## G. Counterparts
| Counterpart | Gap closed by this IP |
|---|---|
| ServiceNow privacy/audit exports | DPIA packet generated from concrete ITSM fields |
| Jira Service Management data residency exports | Pack-specific evidence and refusal behavior |
| Freshservice compliance evidence | Import aliases and processors are documented |

## H. Cold-start buildability notes
- Inventory OpenAPI fields before writing DPIA prose.
- Map every field to a domain data class.
- Keep processors and recipients distinct.
- Use pack fixtures for GDPR, KR-PIPA, and HIPAA.
- Add redaction tests before export bundle work.
- Keep source-system ids as aliases in DPIA evidence.
- Do not claim retention controls without a source artifact.
- Emit audit evidence on DPIA packet export.
- Include unresolved evidence gaps in the packet.
- Preserve links to policy and contract files.

## API Versioning (per ADR-0342)

- contract_surface: [`microservices/itsm/contracts/asyncapi-v1.yaml`, `microservices/itsm/contracts/itsm-v1.proto`, `microservices/itsm/contracts/local-asyncapi-v1.yaml`, `microservices/itsm/contracts/local-openapi-v1.yaml`, `microservices/itsm/contracts/local-operations-v1.proto`, `microservices/itsm/contracts/openapi-v1.yaml`]; detected_types: OpenAPI, AsyncAPI, proto3; trigger_terms: [`openapi`].
- carrier: `YYYY-MM-DD` via header `Oyatie-Version`, URL prefix `/v/<date>/`, and proto3 envelope field tag `8001`.
- declared_version: `2026-05-21`; supported_window: latest `N=3` public date versions for `>=180` days.
- internal_mesh_exemption: internal gRPC remains unaffected per ADR-0145; this section applies at public contract boundaries.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/itsm/IP-023-dpia-evidence-packet.md` matched [`PHI`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/itsm/IP-023-dpia-evidence-packet.md`, `microservices/itsm/manifest.json`, `microservices/itsm/ARCHITECTURE.md`, `microservices/itsm/PRD.md`, `microservices/itsm/multi-region.md`, `microservices/itsm/capacity-model.md`].
