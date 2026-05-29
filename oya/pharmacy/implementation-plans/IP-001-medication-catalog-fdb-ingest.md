# IP-001 — MedicationCatalog kernel + FDB ingest adapter + A/B knowledge package switching

- **Status**: draft
- **Owner**: axis-pharmacy
- **Authority**: ADR-0332, ADR-MS-PHARM-001
- **Depends on**: ADR-0105 13-layer, ADR-0145 inter-microservice gRPC
- **Estimated complexity**: M

## Goal

Stand up the canonical medication identity graph (RxCUI + NDC11 + GPI + ATC + UNII) and an FDB MedKnowledge ingestion adapter with A/B knowledge-package switching for safety rollback.

## Acceptance criteria

- AC-1: `oya-pharmacy-medication-catalog-kernel` defines `Medication`, `Ingredient`, `Package`, `DrugKnowledgePackage` value objects with `RxCUI`, `NDC11`, `GPI`, `ATC`, `UNII` newtypes.
- AC-2: `oya-pharmacy-medication-catalog-domain` provides `normalize_ndc10_to_ndc11`, `link_brand_generic`, `derive_atc_l4`.
- AC-3: `oya-pharmacy-medication-catalog-adapter-fdb` ingests an FDB MedKnowledge monthly drop and writes to PostgreSQL Citus via the `medication-catalog-adapter` port.
- AC-4: `oya-pharmacy-medication-catalog-usecase::switch_knowledge_package(tenant, vendor, version)` performs an atomic A/B switch.
- AC-5: `oya-pharmacy-medication-catalog-rest` exposes `GET /Medication` and `GET /Medication/{id}` per OpenAPI 1.0.
- AC-6: `oya-pharmacy-medication-catalog-worker` runs monthly RxNorm release reconciler.
- AC-7: Unit tests cover NDC normalization, RxCUI linking, ATC class derivation.
- AC-8: Property tests for NDC10 → NDC11 round-trip with leading-zero packing.

## Tasks

1. Define kernel value objects.
2. Implement NDC normalization + ATC L4 classifier in domain.
3. Build FDB ingest adapter with checksumed package validation.
4. Build Multum + Medi-Span adapters parallel.
5. Implement A/B switch usecase.
6. Wire REST endpoints.
7. Wire monthly RxNorm worker.
8. Tests + benches.

## Out of scope

- Multum + Medi-Span deep adapters beyond skeleton (deferred to IP-001-FOLLOWUP).
- Real-time RxNorm streaming.

## Risks

- Knowledge package versioning collisions across tenants → tenant-scoped version registry.
- NDC ambiguity (NDC10 vs NDC11 vs GTIN-14) → strict normalize-on-ingest.
