# IP-010 — Reimbursement 340B + NCPDP D.0 PBM + handoff to cloud-billing

- **Status**: draft
- **Owner**: axis-pharmacy
- **Authority**: ADR-0332
- **Depends on**: IP-001, IP-003, IP-009
- **Estimated complexity**: L

## Goal

Implement 340B eligibility evaluation with auditable evidence, NCPDP D.0 PBM claim submission, reject classification, and handoff to `cloud-billing` for charge posting.

## Acceptance criteria

- AC-1: Kernel: `Claim`, `PBMClaim`, `B340Determination`, `CopayResult`, `RejectCode`.
- AC-2: Domain: NCPDP D.0 envelope codec; 340B mixed-use classifier with encounter-evidence trail.
- AC-3: PBM adapter: outbound D.0 over TLS; rotating creds from OpenBao at `secret/pharmacy/pbm-ncpdp-issuer-credential`.
- AC-4: 340B replenishment lot tagging.
- AC-5: cloud-billing handoff via gRPC `cloud-billing.Charge`.
- AC-6: AsyncAPI `oya.pharmacy.reimbursement.claim-accepted`, `oya.pharmacy.reimbursement.claim-rejected`.
- AC-7: SLO `340b-classification-accuracy` ≥ 99.99% (audit-sample-driven).
- AC-8: Tests covering reject-code classification, 340B mixed-use, copay calc.

## Tasks

1. Kernel + domain.
2. NCPDP D.0 codec.
3. PBM adapter.
4. 340B classifier.
5. cloud-billing handoff.
6. AsyncAPI.
7. SLO metrics wiring.
8. Tests.

## Risks

- Reject-code taxonomy drift (NCPDP versioning) → versioned codec.
- 340B HRSA-OPAIS reporting cadence → quarterly worker.
- PBM contract pricing complexity → table-driven contract model.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/pharmacy/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/pharmacy/implementation-plans/IP-010-reimbursement-340b-pbm.md:21` - - AC-7: SLO `340b-classification-accuracy` ≥ 99.99% (audit-sample-driven).; `microservices/pharmacy/implementation-plans/IP-010-reimbursement-340b-pbm.md:32` - 7. SLO metrics wiring..
