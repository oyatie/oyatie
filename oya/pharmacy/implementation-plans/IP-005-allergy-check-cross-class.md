# IP-005 — AllergyCheck mirror + cross-class derivation + override capture

- **Status**: draft
- **Owner**: axis-pharmacy
- **Authority**: ADR-0332
- **Depends on**: IP-001
- **Estimated complexity**: M

## Goal

Mirror patient allergy lists from `emr`, normalize allergens, derive cross-class matches, capture structured overrides with attestation.

## Acceptance criteria

- AC-1: `oya-pharmacy-allergy-check-kernel` types: `AllergyRecord`, `AllergyCheckResult`, `AllergyMatchKind` (exact-ingredient | class-match | no-match).
- AC-2: `oya-pharmacy-allergy-check-domain` normalizes to RxNorm ingredient + UNII + SNOMED CT substance; derives cross-class via knowledge-graph.
- AC-3: `oya-pharmacy-allergy-check-adapter-emr` reads patient allergy list via gRPC `emr.Allergy`.
- AC-4: `oya-pharmacy-allergy-check-usecase::check(patient, rxcui)` returns finding with override eligibility.
- AC-5: Cedar policy `allergy-override-requires-justification.cedar` enforces severity ≥ severe two-step.
- AC-6: AsyncAPI event `oya.pharmacy.allergy.override-attested` emitted on override.
- AC-7: Tests covering penicillin → cephalosporin class match; sulfa → diuretic class match.

## Tasks

1. Kernel + domain.
2. Allergen normalization (RxNorm + UNII + SNOMED CT).
3. Cross-class derivation.
4. EMR adapter.
5. Override capture + Cedar gate.
6. AsyncAPI emission.
7. Tests.

## Risks

- False-positive class matches → tunable class-match suppression with audit.
- Allergy list freshness from EMR → idempotent incremental mirror with HLC ordering.

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: eligible for non-urgent batch, replay, export, backfill, package, or analytics work when error budget and pack recovery bounds permit deferral.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/pharmacy/implementation-plans/IP-005-allergy-check-cross-class.md:30` - 6. AsyncAPI emission..
