# IP-003 — ePrescribe usecase + Surescripts adapter + EPCS sign envelope

- **Status**: draft
- **Owner**: axis-pharmacy
- **Authority**: ADR-MS-PHARM-001 (ePrescribe substrate)
- **Depends on**: IP-001, IP-002
- **Estimated complexity**: L

## Goal

Build the ePrescribe orchestrator end-to-end: NCPDP SCRIPT 2017-071 codec, Surescripts mTLS client, EPCS signing flow for Schedule II–V via DEA-bound KMS keys.

## Acceptance criteria

- AC-1: `oya-pharmacy-eprescribe-kernel` types covering NCPDP SCRIPT NewRx / RxRenewal / RxChange / CancelRx / RxFill / RxHistory / REMS messages.
- AC-2: `oya-pharmacy-eprescribe-adapter-surescripts` performs mTLS handshake with rotating cert from OpenBao at `secret/pharmacy/surescripts-mtls-*`.
- AC-3: `oya-pharmacy-eprescribe-adapter-epcs-kms` signs EPCS envelope via `cloud-kms` with individual prescriber DEA-bound key.
- AC-4: `oya-pharmacy-eprescribe-usecase::transmit(prescription)` orchestrates DDI/DAI/DCI/DPI/DRC → verification → sign → transmit.
- AC-5: `oya-pharmacy-eprescribe-worker` handles inbound Surescripts messages.
- AC-6: Cedar gate `prescriber-can-eprescribe.cedar` enforced.
- AC-7: Audit-chain seal events `oya.pharmacy.rx.prescribed` and `oya.pharmacy.eprescribe.epcs-signed`.
- AC-8: SLO `eprescribe-roundtrip-latency` (p95 ≤ 5 s) wired with Prometheus metrics.

## Tasks

1. NCPDP SCRIPT codec.
2. Surescripts adapter with cert rotation handler.
3. EPCS KMS adapter.
4. Outbound usecase orchestration.
5. Inbound worker.
6. Cedar policy compile + cache hookup.
7. Audit emission.
8. SLO metrics wiring.

## Risks

- NCPDP version migration during build (low; cadence ~3y).
- Surescripts production endpoint provisioning takes 8–12 weeks per accreditation cycle.
- EPCS DEA registration verification cron must run reliably (covered in IP-003-FOLLOWUP).

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/pharmacy/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `valkey`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/pharmacy/implementation-plans/IP-003-eprescribe-surescripts-epcs.md:22` - - AC-8: SLO `eprescribe-roundtrip-latency` (p95 ≤ 5 s) wired with Prometheus metrics.; `microservices/pharmacy/implementation-plans/IP-003-eprescribe-surescripts-epcs.md:33` - 8. SLO metrics wiring..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: eligible for non-urgent batch, replay, export, backfill, package, or analytics work when error budget and pack recovery bounds permit deferral.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/pharmacy/implementation-plans/IP-003-eprescribe-surescripts-epcs.md:32` - 7. Audit emission..
