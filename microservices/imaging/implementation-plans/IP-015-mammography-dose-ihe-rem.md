# IP-015 — Mammography tracking + radiation dose + IHE REM

`scope: oya-imaging-mammography-tracking-app + oya-imaging-dose-tracking-app + oya-imaging-mammography-recall-worker + IHE REM actors`
`wave_target: 21-imaging-mammography-dose`
`adr_binding: ADR-0251 (MQSA + EU-MDR + EURATOM packs) + ADR-MS-004`

## Objective

Stand up mammography screening recall + BI-RADS audit + MQSA retention + radiation dose tracking + IHE REM (Radiation Exposure Monitoring) actors.

## Scope

1. Mammography screening recall workflow (FR-MAMMO-001).
2. BI-RADS audit: positive predictive value, cancer detection rate, sensitivity, specificity (FR-MAMMO-002).
3. DBT synthesized 2D + sliced 3D display (FR-MAMMO-003).
4. MQSA retention default 10 years US (FR-MAMMO-004).
5. Mammography CAD on synthesized 2D + DBT slices (FR-MAMMO-005).
6. RDSR parsing on C-STORE-completed event (FR-DOSE-001).
7. Per-protocol dose deviation alerts at DLP > target × 1.25 (FR-DOSE-002).
8. Aggregate dose dashboards (FR-DOSE-003).
9. EURATOM 2013/59 dose register export (FR-DOSE-004).
10. CMS QPP MIPS Measure 145 export (FR-DOSE-005).
11. IHE REM actor: Acquisition Modality + RDSR Repository + Dose Information Reporter + Dose Information Consumer.

## Acceptance criteria

- BI-RADS audit numbers correctness test against synthetic cohort.
- MQSA 10-year retention enforced.
- DLP > target × 1.25 alert fires.
- EURATOM export XML validates against EURATOM schema.
- IHE Connectathon REM pass.

## Dependencies

- IP-001, IP-006, IP-009.

## Risks

- Per-protocol target DLP authoring effort; mitigate with ACR reference protocols.
- MQSA inspection compliance.

## Estimated effort

- 10–14 person-weeks.
