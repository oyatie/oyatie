# IP-004 — DrugInteraction eight-engine fan-out + severity bands + tenant suppression

- **Status**: draft
- **Owner**: axis-pharmacy
- **Authority**: ADR-0332
- **Depends on**: IP-001
- **Estimated complexity**: L

## Goal

Build the DDI / DAI / DCI / DPI / DDxI / DLI / DFI / DDoseI engine fan-out with severity stratification and per-tenant suppression.

## Acceptance criteria

- AC-1: `oya-pharmacy-drug-interaction-kernel` defines `InteractionEngine` enum (8 variants), `InteractionSeverity` enum (6 levels), `InteractionFinding` value.
- AC-2: `oya-pharmacy-drug-interaction-domain` implements eight sub-engines with monograph evidence linkage.
- AC-3: `oya-pharmacy-drug-interaction-usecase::evaluate(patient, rxcuis, engines)` runs parallel fan-out.
- AC-4: Per-tenant suppression: severity bands below `severe` may be suppressed; `severe` + `contraindicated` require Cedar override.
- AC-5: SLO `ddi-check-latency` p99 ≤ 200 ms.
- AC-6: AsyncAPI events `oya.pharmacy.rx.alert.ddi`, `.dai`, `.dci`, `.dpi`, `.drc` emitted per finding.
- AC-7: Tests covering false-positive suppression and severity escalation.

## Tasks

1. Kernel + domain engine traits.
2. Eight sub-engine implementations (DDI/DAI/DCI/DPI/DDxI/DLI/DFI/DDoseI).
3. Parallel fan-out usecase.
4. Per-tenant suppression policy resolver.
5. AsyncAPI emission.
6. Cedar gate for severe/contraindicated suppression.
7. Bench (criterion) for p99 ≤ 200 ms validation.

## Risks

- Knowledge package source consistency (FDB vs Multum vs Medi-Span) → vendor adapter selection per tenant.
- Override fatigue → suppression analytics fed back to `analytics` substrate.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/pharmacy/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `iceberg_snapshot`, `clickhouse_iceberg_layered`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/pharmacy/implementation-plans/IP-004-drug-interaction-eight-engine.md:19` - - AC-5: SLO `ddi-check-latency` p99 ≤ 200 ms.; `microservices/pharmacy/implementation-plans/IP-004-drug-interaction-eight-engine.md:31` - 7. Bench (criterion) for p99 ≤ 200 ms validation..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: eligible for non-urgent batch, replay, export, backfill, package, or analytics work when error budget and pack recovery bounds permit deferral.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/pharmacy/implementation-plans/IP-004-drug-interaction-eight-engine.md:29` - 5. AsyncAPI emission..
