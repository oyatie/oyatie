# IP-009 — Structured reporting (DICOM SR + FHIR DiagnosticReport[imaging])

`scope: oya-imaging-report-app + oya-imaging-report-domain + oya-imaging-fhir-rest`
`wave_target: 18-imaging-rad-workflow`
`adr_binding: ADR-0105 + ADR-MS-004`

## Objective

Stand up structured-reporting engine covering BI-RADS, LI-RADS, PI-RADS, TI-RADS, Lung-RADS, O-RADS, CAD-RADS, Bone-RADS, NI-RADS, ACR templates. Emit DICOM SR + FHIR DiagnosticReport[imaging] dual.

## Scope

1. Template library (ACR + RadLex + RSNA RadElement).
2. Report draft / save / sign lifecycle.
3. DICOM SR-TID-1500 emission.
4. FHIR DiagnosticReport[imaging] R5 emission.
5. Save p95 < 800ms (FR-RAD-008).
6. Critical-finding code tagging.

## Acceptance criteria

- All 9 RADS templates load.
- Dual DICOM-SR + FHIR DiagnosticReport emission asserted byte-exact against reference fixtures.
- Save p95 < 800ms.
- Critical-finding tagging triggers IP-011 critical-result flow.

## Dependencies

- IP-001, IP-007, IP-008.

## Risks

- FHIR R5 vs R4 dual support; ship R5 first, R4 adapter follow-up.
- Terminology binding drift (SNOMED-CT releases).

## Estimated effort

- 10–12 person-weeks.

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/imaging/implementation-plans/IP-009-structured-reporting.md:15` - 3. DICOM SR-TID-1500 emission.; `microservices/imaging/implementation-plans/IP-009-structured-reporting.md:16` - 4. FHIR DiagnosticReport[imaging] R5 emission..
