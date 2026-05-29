# IP-007 — Verification + tall-man-lettering + dual-verify CII

- **Status**: draft
- **Owner**: axis-pharmacy
- **Authority**: ADR-MS-PHARM-002 (controlled-substance DEA compliance)
- **Depends on**: IP-001, IP-004, IP-005, IP-006
- **Estimated complexity**: M

## Goal

Pharmacist verification subsystem with single vs. dual verification per schedule, tall-man lettering rendering, alert dismissal capture.

## Acceptance criteria

- AC-1: Kernel: `VerificationTicket`, `DualVerification`.
- AC-2: Domain tall-man lettering function (per ISMP list).
- AC-3: Usecase orchestrates: gather all CDS alerts → render verification view → capture decision + dismissals → emit event.
- AC-4: Cedar `pharmacist-can-verify.cedar` and `dea-controlled-2x-verify.cedar` enforced.
- AC-5: AsyncAPI `oya.pharmacy.rx.verified`.
- AC-6: Tests covering single-pharmacist queueing for CII dual-verify.

## Tasks

1. Kernel + domain.
2. Tall-man lettering renderer.
3. Verification usecase.
4. Cedar wiring.
5. AsyncAPI emission.
6. Tests.

## Risks

- Single-pharmacist facilities + CII dual-verify → witness-queue with SLA.

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: eligible for non-urgent batch, replay, export, backfill, package, or analytics work when error budget and pack recovery bounds permit deferral.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/pharmacy/implementation-plans/IP-007-verification-tall-man-dual-verify.md:28` - 5. AsyncAPI emission..
