# ADR-MS-PHARM-002 — Controlled-substance DEA compliance

- **Status**: Accepted
- **Date**: 2026-05-21
- **Owner**: axis-pharmacy
- **Authority**: ADR-0332 (pharmacy substrate authorization), ADR-0243 (Cedar universal gate), ADR-0251 (compliance pack primitive), ADR-MS-PHARM-001 (ePrescribe substrate)
- **Regulatory**: 21 CFR §1300–§1321 (DEA Controlled Substances Act, Diversion Control)

## Context

The DEA regulates Schedule II–V controlled substances with prescriptive requirements that exceed HIPAA:

- **Schedule II (CII)**: opioids (oxycodone, fentanyl, methadone), stimulants (methylphenidate, amphetamine), cocaine. No refills; EPCS for electronic Rx; DEA Form 222 ordering; perpetual inventory; witnessed waste.
- **Schedule III–V**: lower-abuse potential; allow refills; require EPCS for electronic Rx; perpetual inventory.

Compliance auditors (DEA Diversion Investigators, state board of pharmacy) require:
- A perpetual inventory reconciled to actual count per 21 CFR §1304.
- Witnessed waste with two pharmacist signatures per dose disposed.
- DEA Form 222 (or equivalent CSOS electronic equivalent) for CII purchasing.
- An EPCS-compliant ePrescribing flow when electronic Rx is used.
- Real-time inspection-ready reporting.

## Decision

Pharmacy SHALL build the controlled-substance plane as a first-class bounded context (`controlled-substance`) with the following invariants:

### Invariants

1. **Dual-pharmacist verification for CII** — Cedar policy `dea-controlled-2x-verify.cedar` MUST forbid a dispense without both primary and witness verifier identities distinct and active.
2. **Witnessed waste signatures** — every waste event MUST capture primary + secondary witness identities, both active and licensed in the dispense state.
3. **Perpetual inventory cadence** — per 21 CFR §1304: physical count cadence (currently biennial for federal floor; many state boards demand monthly or quarterly). Cell job runs `controlled-substance-worker.reconcile()` on configured cadence.
4. **DEA Form 222 / CSOS** — CII ordering MUST flow through `controlled-substance-usecase.submit_form222()` with prescriber/buyer DEA signatures.
5. **EPCS signing** — per ADR-MS-PHARM-001, individual DEA-bound KMS keys; no shared platform key.
6. **Audit-chain bilateral** — every controlled event sealed in `audit-chain` with bilateral cross-pointer for cross-tenant flows (e.g., specialty hub).
7. **DEA inspection report** — `GenerateDEAInspectionReport` MUST be available on demand with no more than 1 hour to generate a complete period report.

### Cell capability tags

- `dea-controlled-substance-vault` — cells holding CII–CV physical inventory MUST advertise this tag.
- `iso-7-negative-pressure` — required for USP 800 hazardous-drug compounding (see ADR-MS-PHARM-003 future).

### Identity binding

- Pharmacists handling CII MUST have active state license + active DEA-registered facility binding.
- Witness pharmacist MUST be a distinct active pharmacist (Cedar policy enforces `primary_verifier_id != witness_verifier_id`).

### State overlays

DEA federal floor is overlaid by state board of pharmacy rules. The `state-board-of-pharmacy` compliance pack provides per-state overlay (CA, TX, NY, FL, etc.) with stricter cadence, additional witness rules, or extended retention.

## Consequences

### Positive
- DEA Diversion Control auditors can trace every controlled-substance hop end-to-end.
- State board of pharmacy inspections satisfied with no special preparation.
- 340B replenishment-lot tagging interoperates with controlled-substance flow.

### Negative
- Dual-pharmacist verification adds latency in single-pharmacist facilities; queue-and-witness flow required.
- KMS-individual key management is operationally heavier than platform-shared.
- DEA registration verification must be operated out-of-band on monthly cron.

## Alternatives considered

- **Single-pharmacist verification for CII** — rejected; federal floor demands two-person verification for CII waste, and most state boards interpret this to extend to verification.
- **Shared KMS key for EPCS** — rejected as a non-starter per DEA registration rules.
- **Outsource controlled-substance ledger to a broker** — rejected; sovereignty and audit-chain integrity, same as ADR-MS-PHARM-001.

## Related ADRs

- ADR-MS-PHARM-001 ePrescribe substrate
- ADR-0243 Cedar universal gate
- ADR-0251 compliance pack primitive
- ADR-0332 pharmacy substrate authorization

## References

- 21 CFR §1300 (definitions)
- 21 CFR §1301 (registration)
- 21 CFR §1304 (records)
- 21 CFR §1306 (prescriptions)
- 21 CFR §1308 (schedules)
- 21 CFR §1311 (electronic prescriptions / EPCS)
- 21 CFR §1317 (disposal)
- 21 CFR §1321 (delegation of authority)
- DEA Diversion Control Division publication: "Practitioner's Manual"
