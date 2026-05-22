---
doc_class: IP
ip_id: IP-015-data-residency-pack-overlays
microservice: itsm
status: rewritten-wave-15-ip-substance
date: 2026-05-21
owner_team: axis-itsm + compliance
counterparts: [ServiceNow ITSM, Jira Service Management, Freshservice]
source_artifacts:
  - microservices/itsm/policy/data-residency.md
  - microservices/itsm/manifest.json
  - microservices/itsm/src/domain/mod.rs
  - microservices/itsm/contracts/openapi-v1.yaml
---

# IP-015 ITSM Data Residency Pack Overlays

## A. Problem
ITSM records include incident narratives, requester identity, CI topology, change evidence, KB articles, and postmortems. A generic tenant id check is not enough when GDPR, KR-PIPA, HIPAA, FedRAMP-High, or SOC-2 packs change retention, export, or display behavior.

The stamped IP did not identify data classes or cross-region actions. This IP ties residency overlays to ITSM's real records.

## B. Approach
Use `policy/data-residency.md` and the manifest compliance pack list to define higher-restriction-wins behavior:

| Data | Residency-sensitive action |
|---|---|
| incident ticket notes | cross-cell read/export |
| change evidence | audit export and retention |
| CMDB relation | service-map traversal across region |
| KB article | RAG retrieval and portal display |
| incident-room transcript | postmortem handoff/export |

## C. Deliverables
- Data-class mapping from `DataClass` in `src/domain/mod.rs` to pack overlay behavior.
- Policy checks before status publication, KB retrieval, CMDB relation traversal, and export.
- Tests that restricted packs block cross-region reads and redact public status output.
- Dashboard evidence for residency denies.
- Documentation of demo_trial pack restrictions from `manifest.json`.

## D. Implementation
1. Extend request context with `jurisdiction_code`, `home_cell`, `pack_ids`, and `data_class`.
2. Map `DataClass::{SupportConfidential,OperationalTelemetry,ChangeEvidence,AuditEvidence}` to pack rules.
3. Apply the resolver before returning ticket body, CI relation, KB article, or postmortem export.
4. For public status updates, redact internal CI names and requester details before publish.
5. For cross-cell recovery, allow metadata-only replication unless pack permits full body movement.
6. Add tests for HIPAA/PHI redaction and KR-PIPA/GDPR residency block.
7. Emit `pack_residency_block` audit evidence on refusal.
8. Keep denial bodies redacted: no object existence leaks across tenant or region.

## E. Acceptance
- Restricted packs deny cross-region body reads for incident, KB, CMDB, and postmortem data.
- Public status output contains no internal CI or requester PII when pack rules require redaction.
- Demo_trial cannot activate blocked compliance packs listed in `manifest.json`.
- Every residency denial is audit-visible.

## F. Evidence
- `policy/data-residency.md` is the ITSM residency policy surface.
- `manifest.json` lists SOC-2, ISO-27001, ITIL, GDPR, KR-PIPA, FedRAMP-High, and HIPAA.
- `src/domain/mod.rs` defines ITSM `DataClass`.
- ADR-0244 and ADR-0251 govern tenant and compliance-pack behavior.

## G. Counterparts
| Counterpart | Gap closed by this IP |
|---|---|
| ServiceNow data residency / domain separation | Pack overlays enforce residency before read/export |
| Jira Service Management Enterprise residency | Project-like boundaries do not replace tenant/cell gates |
| Freshservice regional hosting | Data-class-specific redaction and denial evidence |

## H. Cold-start buildability notes
- Map domain `DataClass` values before adding new contract fields.
- Add redaction tests for status updates before export tests.
- Treat pack conflict as deny with audit evidence.
- Do not replicate incident-room bodies by default.
- Keep demo_trial pack blocks aligned with manifest.
- Use synthetic GDPR, KR-PIPA, and HIPAA tenants in tests.
- Avoid returning object existence in residency-denied responses.
- Keep public status output separate from operator notes.
- Add cell labels to audit evidence.
- Record missing pack resolver APIs as follow-up rather than inventing them.

## API Versioning (per ADR-0342)

- contract_surface: [`microservices/itsm/contracts/asyncapi-v1.yaml`, `microservices/itsm/contracts/itsm-v1.proto`, `microservices/itsm/contracts/local-asyncapi-v1.yaml`, `microservices/itsm/contracts/local-openapi-v1.yaml`, `microservices/itsm/contracts/local-operations-v1.proto`, `microservices/itsm/contracts/openapi-v1.yaml`]; detected_types: OpenAPI, AsyncAPI, proto3; trigger_terms: [`openapi`].
- carrier: `YYYY-MM-DD` via header `Oyatie-Version`, URL prefix `/v/<date>/`, and proto3 envelope field tag `8001`.
- declared_version: `2026-05-21`; supported_window: latest `N=3` public date versions for `>=180` days.
- internal_mesh_exemption: internal gRPC remains unaffected per ADR-0145; this section applies at public contract boundaries.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/itsm/IP-015-data-residency-pack-overlays.md` matched [`PHI`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/itsm/IP-015-data-residency-pack-overlays.md`, `microservices/itsm/manifest.json`, `microservices/itsm/ARCHITECTURE.md`, `microservices/itsm/PRD.md`, `microservices/itsm/multi-region.md`, `microservices/itsm/capacity-model.md`].
