# IP-006 — DoseCheck weight/BSA/renal/hepatic/age-band/cumulative

- **Status**: draft
- **Owner**: axis-pharmacy
- **Authority**: ADR-0332
- **Depends on**: IP-001
- **Estimated complexity**: M

## Goal

Implement dose range checking: weight-based, BSA-based, renal-adjusted (eGFR CKD-EPI + CrCl Cockcroft-Gault), hepatic-adjusted (Child-Pugh), age-banded, cumulative caps.

## Acceptance criteria

- AC-1: Kernel types `DosingRule`, `DoseCheckResult`, with per-organ-band predicates.
- AC-2: Domain compute: `eGFR_CKD_EPI(age, sex, race, scr)`, `CrCl_Cockcroft_Gault(age, weight, sex, scr)`, `Child_Pugh(albumin, bilirubin, INR, ascites, encephalopathy)`.
- AC-3: Pediatric weight-based dosing never defaults to adult dosing (hard fail).
- AC-4: Stale renal value (> 72 h) flagged.
- AC-5: Cumulative lifetime caps tracked for anthracyclines etc.
- AC-6: REST `/Dose/check`.
- AC-7: Tests covering pediatric, geriatric, renal-impairment, hepatic-impairment edge cases.

## Tasks

1. Kernel + domain computations.
2. Dosing rule loader from knowledge package.
3. Stale-value detection.
4. Cumulative-cap state.
5. REST.
6. Tests.

## Risks

- eGFR equation variants (CKD-EPI 2009 vs 2021) → tenant configuration.
- Race coefficient debate → 2021 equation default (race-neutral).
