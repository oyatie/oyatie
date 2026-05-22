---
doc_class: IP
ip_id: IP-020-catalog-layer-registration
microservice: itsm
status: rewritten-wave-15-ip-substance
date: 2026-05-21
owner_team: axis-itsm + catalog
counterparts: [ServiceNow ITSM, Jira Service Management, Freshservice]
source_artifacts:
  - microservices/itsm/manifest.json
  - microservices/itsm/src/lib.rs
  - microservices/itsm/PRD.md
  - microservices/itsm/competitor-parity-matrix.md
---

# IP-020 ITSM Catalog Layer Registration

## A. Problem
ITSM must be discoverable as an Oyatie product capability without becoming a suite island. The stamped IP did not define what gets registered: service descriptor, bounded contexts, contracts, compliance packs, SLOs, or counterpart coverage.

This IP registers ITSM with the catalog layer as a flat µservice that exposes five bounded contexts and multiple tenant-facing capability surfaces.

## B. Approach
Use the live `descriptor()` in `src/lib.rs` and `manifest.json` as registration sources:

| Catalog field | Source |
|---|---|
| microservice slug | `MICROSERVICE` |
| bounded contexts | `BOUNDED_CONTEXTS` |
| contracts | `OPENAPI_CONTRACT`, `GRPC_CONTRACT`, `ASYNCAPI_CONTRACT` |
| owner | `manifest.json` owner_team |
| counterparts | manifest top_3_counterparts / parity matrix |
| packs | manifest compliance_packs |

## C. Deliverables
- Catalog registration record referencing ITSM contracts and bounded contexts.
- Validation that `descriptor()` and `manifest.json` agree on service slug and contracts.
- Counterpart rows for ServiceNow ITSM, Jira Service Management, and Freshservice.
- Links from catalog entry to PRD, architecture, remediation notes, and parity matrix.
- Tests or gate output proving flat layout and layer enum conformance.

## D. Implementation
1. Read `descriptor()` and `manifest.json` as the two sources for registration.
2. Register `itsm` as product-critical umbrella µservice, not a product suite.
3. Include bounded contexts: on-call-schedule, escalation-policy, incident-room, status-update, postmortem.
4. Include contract paths and capability names from OpenAPI enum where applicable.
5. Include compliance packs and tenant-class behavior.
6. Include counterpart links to ServiceNow/Jira/Freshservice parity evidence.
7. Add a validation that descriptor contract count remains 3.
8. Make rollback remove the catalog row only; do not delete service artifacts.

## E. Acceptance
- Catalog row points to real files in `microservices/itsm/`.
- `validate_scaffold()` continues to pass.
- Catalog does not create separate microservices for ServiceNow, Jira, or Freshservice.
- Counterpart coverage is visible from the registration row.

## F. Evidence
- `src/lib.rs` defines `ServiceDescriptor`, `descriptor()`, and `validate_scaffold()`.
- `manifest.json` lists owner, top-3 counterparts, dependencies, packs, and layer conformance.
- `PRD.md` documents user stories and functional requirements.
- ADR-0131 governs flat layout; ADR-0316 governs capability-tier discipline.

## G. Counterparts
| Counterpart | Gap closed by this IP |
|---|---|
| ServiceNow ITSM product catalog identity | ITSM is registered without suite fragmentation |
| Jira Service Management marketplace/category presence | Catalog row links capability and contract evidence |
| Freshservice ITSM product surface | Tenant sees packs, contracts, and SLO readiness |

## H. Cold-start buildability notes
- Use `descriptor()` as a verification source, not just manifest prose.
- Keep top-3 counterparts visible in the catalog row.
- Register bounded contexts exactly as `BOUNDED_CONTEXTS`.
- Include three contract paths and fail if contract count changes unexpectedly.
- Do not register ServiceNow/Jira/Freshservice as separate microservices.
- Link PRD and architecture as evidence, not copied summaries.
- Keep compliance pack list from manifest.
- Add flat-layout gate evidence when available.
- Roll back only the catalog row.
- Treat missing catalog schema as a follow-up.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/itsm/IP-020-catalog-layer-registration.md` matched [`SLO`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/itsm/IP-020-catalog-layer-registration.md`, `microservices/itsm/manifest.json`, `microservices/itsm/ARCHITECTURE.md`, `microservices/itsm/PRD.md`, `microservices/itsm/multi-region.md`, `microservices/itsm/capacity-model.md`].
