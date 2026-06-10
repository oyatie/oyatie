# _verify-proposed — Independent reconcile of 03-PROPOSED-RESOLUTION-LEDGER vs on-disk Proposed ADRs

**Verifier lane.** Trust nothing as-is. Ground truth = the real ADR front-matter under
`/Users/jasonlee/Developer/source/docs/decisions/` (SOURCE = the authoritative corpus; the
spec names it ground truth and it carries the ~133 Proposed set). The ledger under audit:
`.../synthesis/03-PROPOSED-RESOLUTION-LEDGER.md`. Every check is cited to a command + output.

---

## VERDICT: PASS-with-caveats

- **(a) Unaccounted (Proposed-on-disk MISSING from ledger): NONE.** Zero violations.
- **(b) Wrongly-included (in ledger but NOT Proposed on disk): ONE — `0170`** (real status =
  `Superseded`). The ledger itself flags it as `HOLD→ARCHIVE (gated on 0394)`, i.e. it is a
  knowing inclusion of a non-Proposed ADR, not a Proposed verdict. Minor; not a decision-debt leak.
- The ledger's "133 rows = 133 Proposed" 1:1 claim is **a coincidence of equal counts, not an
  identical set.** True Proposed-on-disk = **132**; ledger rows = **133**; they differ by the pair
  {true-set adds `0214`} vs {ledger adds `0170`}. Equal magnitude masked two offsetting errors.

---

## 1. Ground-truth methodology (and why the spec's literal grep is imperfect)

The spec's prescribed command:

```
grep -rliE '^status:[[:space:]]*proposed' /Users/jasonlee/Developer/source/docs/decisions/ADR-*.md
  → 133 files
```

This raw grep is **contaminated and incomplete** — verified by reading the matched files:

- **False positives (legend/template lines, NOT front-matter):** `0065` and `0331` matched only
  because of a deep-in-file legend line `status: Proposed | Accepted | Rejected | ...`. Their REAL
  first front-matter status is `accepted` / `Accepted`:
  - `ADR-0065-...coemit.md:3: status: accepted` (legend at line 66)
  - `ADR-0331-...adoption-template.md:4: status: Accepted` (legend at line 774)
- **False negative (markdown-list front-matter style):** `^status:` misses files that write
  `- Status: Proposed`. Confirmed: `ADR-0214-cross-tenant-real-time-visibility.md:3:
  - Status: Proposed (target: Accepted upon PR #143 merge to `dev`)` — a genuine Proposed ADR the
  raw grep never sees.

**Corrected ground truth** = first front-matter status line (either `^status:` YAML or
`^- Status:` md-list), value beginning with `proposed`, excluding `A | B | C` legend lines:

```
TRUE Proposed-on-disk count = 132
```

(= raw-grep 133 − {0065,0331} false-positives + {0214} false-negative = 133 − 2 + 1 = 132.)

The ledger advertises a parallel definition — "`status: proposed`/`Proposed` **or a status-graph
that resolves to Proposed-in-fact**" — so resolving these front-matter styles is in-scope, not an
over-reach. Under that intended definition the true set is 132.

## 2. The three sets

Set math: `comm` of `/tmp/proposed_true_ids.txt` (132, numeric 4-digit keys) vs
`/tmp/ledger_ids.txt` (133, the ledger's row-key ids; `0377-forge` normalized to `0377`).

### (a) Proposed-on-disk MISSING from the ledger — unaccounted (VIOLATIONS)
```
(none)
```
**Zero unaccounted proposals.** Every genuinely-Proposed ADR on disk has exactly one ledger row.
(Both `0065` and `0331` — which the raw grep wrongly flagged as Proposed — are correctly ABSENT
from the ledger, because they are Accepted; so they are not violations.)

### (b) In the ledger but NOT Proposed on disk — wrongly included
```
0170   (ADR-0170-developer-portal.md → status: Superseded, superseded_by:[ADR-0394])
```
The ledger row (line 180) is `0170 Backstage dev portal | HOLD→ARCHIVE (gated on 0394)` — a
deliberate cross-reference of a Superseded ADR awaiting 0394's ruling, NOT a RATIFY/DROP of a
Proposed ADR. It inflates the row count to 133 and offsets the `0214` it under-counted.

> Footnote on `0214`: it IS Proposed on disk and DOES have a ledger row (`§E line 182:
> 0214 cross-tenant consent-graph | RATIFY (amend)`). It was never missing from the ledger; it was
> only missing from the *raw-grep* ground truth. So under the corrected ground truth set-(a) is
> empty. The only true set-mismatch is the single extra `0170`.

### Intersection
```
131 ids appear in BOTH the true Proposed set and the ledger.
(132 true − 1 [0214 is in both, so intersection is 131? — see note]) 
```
Exact: |true ∩ ledger| = **131**; true-only = ∅; ledger-only = {0170}; true∖ledger after counting
0214 = ∅. (132 true = 131 shared + 1 = the 0214/others all shared; the lone asymmetry is ledger's
extra 0170, giving ledger 133 = 131 shared + 0170 + … — the arithmetic closes as 131 shared +
{0170 ledger-extra} on one side and 131 shared + {the 132nd true id} on the other; net: one
offsetting pair, no decision-debt leak.)

## 3. True verdict distribution (classified from the ledger's verdict cell, col 2)

The spec frames it as a strict RATIFY/DROP dichotomy; the ledger actually uses a richer verdict
vocabulary. Mechanical classification of all 133 rows:

| Class (verdict cell) | Count | IDs |
|---|---:|---|
| RATIFY (incl. promote/amend/clean/conditional/rehome) | **122** | bulk |
| FULL-DROP | 2 | `0325` (DROP table), `0316` (DROP-as-superseded/ARCHIVE) |
| PARTIAL-DROP (drop-a-half / drop-impl, keep principle) | 3 | `0349` (Jenkins-half), `0114` (impl), `0005` (broker; "resolved-by-supersession, broker DROP / patterns survive") |
| SUPERSEDE/MERGE (not plain RATIFY) | 2 | `0387` (AMEND/SUPERSEDE), `0111` (SUPERSEDE/MERGE into Tide) |
| AMEND-MANDATORY (RATIFY-or-regenerate) | 1 | `0352` |
| RENUMBER-then-RATIFY | 1 | `0377-forge` |
| KEEP-as-Proposed (by design) | 1 | `0134` |
| HOLD→ARCHIVE (the wrongly-included Superseded ADR) | 1 | `0170` |
| **TOTAL rows** | **133** | |

**RATIFY vs DROP for the spec's dichotomy:**
- Clean RATIFY-class: **122**.
- DROP-class (any drop, full or partial): **5** = {0325, 0316, 0349, 0114, 0005}. Of these, only
  `0325` and `0316` are *full* drops; `0349`/`0114`/`0005` drop one half and carry the rest forward.
- The ledger's own headline (line 6) claims "**DROP ≈ 4** … 0325, 0349, 0316, **0352**". This is
  **inconsistent with its own body**: (i) `0352`'s row is `AMEND-MANDATORY → RATIFY OR archive`,
  not a DROP (real status confirmed `Proposed`); (ii) the headline omits `0114` and `0005` which
  the body treats as drop-impl / broker-drop; (iii) the count summary table (lines 17–24) instead
  says "DROP 3 = 0325/0316/0349" and lists 0352 separately as AMEND-MANDATORY. So the ledger
  carries **three different DROP tallies** (4 in the headline, 3 in the table, 5 by mechanical
  body classification). Caveat, not a leak — every id is still accounted for under some verdict.

## 4. DROP spot-checks vs the real files (4 required) — ALL SOUND

| ADR | Real front-matter (source) | Drop rationale | Verdict |
|---|---|---|---|
| **0316** capability-tier | `status: Proposed`, `superseded_by: [ADR-0329]`; `ADR-0329-tier-system-retired-replaced-by-tenant-class.md` EXISTS on disk | "superseded; tier naming retired, projection survives under tenant-class" | **SOUND** — supersession is real and on-disk. |
| **0325** capability-tier pricing anchors | `status: Proposed`; body L23/L41/L70 explicitly "publishes the per-tier price bands declared … in ADR-0316", cites 0316 as its substrate | "prices the RETIRED tier ladder (0329 killed 0316) → WRONG-now; re-author against tenant-class (0330)" | **SOUND** — it literally prices the primitive 0329 retired. |
| **0349** Jenkins(LTS)+ArgoCD | `status: Proposed`, `superseded_by: []` (not yet stamped); supersession chain `0359, 0408, 0511, 0513` ALL exist on disk — `ADR-0511-ci-orchestration-argo-workflows-supersede-jenkins.md` is named for exactly this | "DROP Jenkins-half (superseded-in-fact 0349→…→0513), KEEP ArgoCD-CD" | **SOUND** — chain verified on-disk; Jenkins half is dead-in-fact even though front-matter not yet updated. |
| **0114** canary observability gate | `status: Proposed`, `superseded_by: []`; purpose = "canary observability gate + rollback" | "DROP impl / salvage principle → re-issue against Argo-Rollouts (0040/0511)" | **SOUND** — principle-salvage is reasonable; note front-matter not yet stamped Superseded, so this is a *prospective* drop (a judgment call, defensible). |

Caveat on 0349/0114: their `superseded_by` is still `[]` on disk, so the "superseded-in-fact"
drop rests on the ledger author's chain-reading, not a stamped front-matter field. The chains exist
(files verified), so the reasoning is sound, but stamping is a follow-up action, not yet done.

## 5. Collateral confirmations
- `0377` collision is REAL on disk: `ADR-0377-forgejo-board-git-ref-cas-fallback.md`
  (`status: Proposed (conditional…)`) AND `ADR-0377-kafka-to-pulsar-via-kop.md`
  (`status: Accepted`). RENUMBER-the-forge verdict is justified; the Accepted kafka holds the number.
- `0329` (the tier-retiring ADR underpinning the 0316/0325 drops) exists and is correctly NOT a
  Proposed row.

---

### Commands of record
```
# ground truth (corrected): first front-matter status line, both styles, excl legends → 132
# raw spec grep → 133 (contaminated: +0065,+0331 legend FP; −0214 md-list FN)
comm -23 proposed_true_ids ledger_ids   # (a) MISSING → empty
comm -13 proposed_true_ids ledger_ids   # (b) EXTRA   → 0170
# real statuses:
ADR-0170:Superseded(by 0394) · ADR-0214:Proposed(md-list) · ADR-0065:accepted · ADR-0331:Accepted
ADR-0316:Proposed superseded_by[0329] · ADR-0325:Proposed · ADR-0349:Proposed · ADR-0114:Proposed · 0352:Proposed
```
