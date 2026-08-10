# app/emr reorg drain notes (`integ/emr`)

## Ownership (rule 3e)

- **Forever home:** `app/emr/**` (this rail).
- **Source:** deleted on trunk by MERGED #1611; reclaim from pre-delete SHA `4aa692919b12`.
- **Writes:** only under `app/emr/**` on this tip.
- **NOT** `integ/healthcare` / `app/healthcare/emr/**` (OVERRULE envelopes 1.16.11 rule 3d).
- **NOT** `integ/app` (composition glue only).

## Completed (this rail)

- Wave-1 absorb: product scaffold reclaim `app/emr/**` → `app/emr/**` from `4aa692919b12`.
- Judgment: envelopes **1.16.11** per-product rail (OVERRULE 1.16.9 nest + #1611 delete).

## Remaining

- Shrink N/A on trunk (already deleted). Do not resurrect under `oya/`.
- Hub retargets on tip-free `integ/specs` only.
- PARKED — no merge to `dev` until wave review.

- `AUDIT-FINDINGS-*.json` excluded (dump-class; not forever face).
