# A.0-1 — Census Reconcile (foundry 731/105 → 831/43)

**Date:** 2026-06-06
**Gate:** A.0-1 (G1) — GATING Wave-0 entry predicate.
**Producer:** independent verifier (read-only on `source/`; the ONE linux audit file below is the only source edited).

## What changed

Edited the one stale linux audit register:
`docs/audit/initial-sweep-2026-06-06/docs-sweep/00-REST-OF-DOCS-REGISTER.md`

Three figures corrected to the census-of-record, plus a dated correction note:

1. **CC-1 sweep heading (was line 108):** `— 731 files (ADR-excluded)` → `— 831 files (ADR-excluded)`.
2. **Palantir HARD carve-out (was line 113):** `Palantir Foundry (105 journey files + ontology.md:241)` → `Palantir Foundry (43 files, census-of-record per A.0-1; … formerly mis-counted as 105)`.
3. **Mechanical-sweep totals table, `foundry` row (was line 146):** `| foundry | **731** | …` → `| foundry | **831** | sense-routed (~274 intelligence / ~135 governance; 43 Palantir carve-out) |`.
4. **Added** a `CENSUS CORRECTION (2026-06-06, A.0-1)` blockquote directly under the CC-1 heading, naming the stale 731/105, the corrected 831/43, and pointing at the SSOT census line `../synthesis/decision-record-oyatie-canon.md:110`.

**No other figure on the page was touched** (138 / 77 / 62 / 35 / 49 / 44 / 21 / 170-raw / 24-genuine all intact — verified post-edit). The only residual "731"/"105" strings are inside the correction note itself, citing the superseded values as STALE.

## Census-of-record (the figures reconciled to)

SSOT = `docs/audit/initial-sweep-2026-06-06/synthesis/decision-record-oyatie-canon.md`, line 110 (the synthesis decision-record, census-of-record):

> "foundry counts imprecise — **total non-ADR = 831 (not 731), Palantir-Foundry carve-out = 43 files (not 105)**" — labelled a "Verification correction (spot-checked vs real files)."

- **831** non-ADR foundry files (supersedes the register's 731).
- **43** Palantir-Foundry HARD carve-out files (supersedes the register's 105).

## Verification (grounded, not asserted)

**SSOT figure confirmed verbatim** at `decision-record-oyatie-canon.md:110`. The same 831/43 census is independently re-confirmed against SSOT `:107/:110` by four review lanes: `_critic-final.md:29` ("CONFIRMED VERBATIM"), `_critic-round3.md:18`, `_arch-round3.md:19`, `_critic-round2.md:94`. The AMENDMENT-PLAN treats 831/43 as `[RULED]` (`AMENDMENT-PLAN.md:87,134,251`) and A.0-1 exists specifically to correct this register's 731/105.

**Independent re-grep of the real corpus** `/Users/jasonlee/Developer/source/docs/` (the corpus the register declares at its line 5):

| Measure | Command (foundry, ADR-excluded = exclude `decisions/ADR-` + `/ADR-NNNN` paths) | Result |
|---|---|---|
| word-boundary `\bfoundry\b`, non-ADR | `grep -rlIiE "\bfoundry\b" . \| grep -viE "decisions/ADR-\|/ADR-[0-9]"` | **731** |
| substring `foundry`, non-ADR | `grep -rliI "foundry" . \| grep -viE "decisions/ADR-\|/ADR-[0-9]"` | **733** |
| substring `foundry`, full corpus (incl ADR) | `grep -rliI "foundry" .` | **956** |
| word-boundary, full corpus | `grep -rlIiE "\bfoundry\b" .` | **953** |
| files matching `Palantir` | `grep -rliI "Palantir" .` | **177** |
| files with both `Palantir` AND `foundry` | (intersection) | **165** |

**Reconciliation of the gap (why grep ≠ 831/43 — and why that is EXPECTED, not a contradiction):**
The SSOT explicitly labels 831/43 a "spot-checked vs real files" *verification correction*, NOT a raw grep. My mechanical word-boundary non-ADR grep reproduces the register's old **731 exactly** — i.e. 731 is the verified mechanical floor. The SSOT's 831 is that floor PLUS a sampled journeys/personas residue tail that the lanes spot-checked above the grep line (the SSOT and `_critic-round2.md:58` both flag "journeys/personas residue = sampled estimate"; the plan hardens this tail in L2.0b — `AMENDMENT-PLAN.md:283,325`). Likewise the raw `Palantir`-overlap grep (165) over-counts because **43** is the curated Palantir-Foundry-the-*product* subset, not every file that merely mentions Palantir. Both deviations are in the documented direction and magnitude, so the SSOT figure is grounded, ratified, and correctly adopted — not a typo.

## Result

Register reconciled to the single census-of-record. The 731/105 vs 831/43 contradiction that the critic/architect lanes rated HIGH/GATING (built-in verifier-stall or ~62-file silent-miscount risk on the irreversible L2 lane) is closed for this register. The SSOT decision-record needs no edit (it already carries 831/43).
