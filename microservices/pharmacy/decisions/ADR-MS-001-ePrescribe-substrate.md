# ADR-MS-PHARM-001 — ePrescribe substrate (Surescripts + NCPDP SCRIPT + EPCS)

- **Status**: Accepted
- **Date**: 2026-05-21
- **Owner**: axis-pharmacy
- **Authority**: ADR-0332 (pharmacy substrate authorization), ADR-0145 (inter-microservice direct gRPC), ADR-0243 (Cedar universal gate), ADR-0244 (tenant scoping), ADR-0251 (compliance pack primitive)

## Context

Pharmacy must originate, receive, and acknowledge electronic prescriptions across the US ePrescribing ecosystem. The de-facto network is Surescripts, layered atop NCPDP SCRIPT 2017-071 (the FHIR-aligned subset is gaining ground but is not yet broadly adopted at production volumes). Controlled-substance prescriptions (DEA Schedule II–V) require EPCS (Electronic Prescribing of Controlled Substances), a DEA-regulated identity + two-factor + signature flow.

Three competing approaches were on the table:

1. **In-house adapter + DEA-individual KMS binding** — own the NCPDP wire format, run a per-prescriber DEA-bound KMS key for EPCS, talk directly to Surescripts over mTLS.
2. **Third-party EPCS broker** — outsource the ePrescribing + EPCS plane to a hosted broker (e.g., DrFirst Rcopia, Imprivata).
3. **Hybrid** — own the message orchestration but outsource EPCS signing to a broker.

## Decision

Adopt Approach 1: **in-house adapter + DEA-individual KMS binding**, with the substrate strictly modular so a broker (Approach 2/3) is a per-tenant configuration switch and never a code change.

Rationale:

- **Sovereignty + lock-in avoidance**: a broker introduces a single-vendor cargo-cult shape; our `feedback_multi_context_provider_agnostic_2026_05_20` directive demands provider-agnostic substrate.
- **Auditability**: DEA EPCS audits scrutinize the chain-of-custody for the signing key. Owning the KMS binding via `cloud-kms` keeps every signature verifiable in our own audit chain.
- **Latency**: brokers add 200–400 ms median to outbound. Our SLO target is p95 ≤ 5 s round-trip; broker adoption is not a bottleneck but ownership reduces it.
- **Substrate fit**: NCPDP SCRIPT 2017-071 is sufficiently stable; an adapter is bounded work.

### Required substrate

- `oya-pharmacy-eprescribe-adapter-surescripts` — Surescripts mTLS client; certificate material in OpenBao at `secret/pharmacy/surescripts-mtls-*`; 90-day rotation with 7-day overlap.
- `oya-pharmacy-eprescribe-adapter-epcs-kms` — DEA-bound KMS signing client. Each prescriber's DEA-bound key MUST be its own KMS material; platform-shared keys are forbidden.
- `oya-pharmacy-eprescribe-domain` — NCPDP SCRIPT 2017-071 message model with deterministic A/B-versioned codecs.
- `oya-pharmacy-eprescribe-usecase` — orchestrates outbound NewRx / RxRenewal / RxChange / CancelRx / RxFill / RxHistory / REMS, inbound message dispatch.
- `oya-pharmacy-eprescribe-worker` — handles asynchronous Surescripts inbound, replay, retry.
- Cedar gates per `prescriber-can-eprescribe.cedar`.
- Audit-chain seal event `oya.pharmacy.eprescribe.epcs-signed`.

### Identity binding

EPCS signing requires:
- Active state license.
- Active DEA registration (verified out-of-band against DEA Diversion Control number registry, refreshed monthly).
- KMS-key binding to the individual prescriber (verifiable in audit chain).
- Two-factor evidence captured at sign time from `identity` step-up (per FIPS 201 / NIST SP 800-63B).

### Failure modes

- Surescripts outbound endpoint down → queue with order-preserving partition per (prescriber, patient), alert at 5 min depth, replay on recovery.
- DEA registration lapsed → block EPCS signing; allow non-controlled prescribing if state license still active; alert prescriber.
- KMS unavailable → fail closed for EPCS; non-controlled prescribing unaffected.
- Two-factor evidence missing → fail closed for EPCS.

## Consequences

### Positive
- Our own EPCS audit chain.
- No broker contract dependency for the controlled-substance plane.
- Per-prescriber key binding satisfies DEA Diversion Control auditors.

### Negative
- We carry the burden of NCPDP SCRIPT version migration (manageable; the version cadence is ~3 years).
- We must operate a Surescripts production endpoint with high reliability.
- DEA registration verification is out-of-band and must be operated as a monthly cron under `governance`.

## Alternatives considered

- **Approach 2 — third-party broker**: rejected for sovereignty and audit-chain integrity reasons. Available as a per-tenant alternative via configuration.
- **Approach 3 — hybrid**: rejected for the same audit-chain reasons; complexity equal to Approach 1 without the sovereignty benefit.

## Related ADRs

- ADR-0145 inter-microservice direct gRPC
- ADR-0243 Cedar universal gate
- ADR-0251 compliance pack primitive
- ADR-0332 pharmacy substrate authorization

## References

- NCPDP SCRIPT Implementation Guide 2017-071.
- 21 CFR §1311 (DEA EPCS).
- FIPS 201 / NIST SP 800-63B.
- Surescripts EHR Vendor Accreditation.
