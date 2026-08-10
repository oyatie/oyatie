# app/healthcare-integration reorg drain notes (`integ/healthcare-integration`)

## Ownership (rule 3e)

- **Forever home:** `app/healthcare-integration/**` (this rail).
- **Source:** deleted on trunk by MERGED #1611; reclaim from pre-delete SHA `4aa692919b12`.
- **Writes:** only under `app/healthcare-integration/**` on this tip.
- **NOT** `integ/healthcare` / `app/healthcare/healthcare-integration/**` (OVERRULE envelopes 1.16.11 rule 3d).
- **NOT** `integ/app` (composition glue only).

## Completed (this rail)

- Wave-1 absorb: product scaffold reclaim `app/healthcare-integration/**` → `app/healthcare-integration/**` from `4aa692919b12`.
- Judgment: envelopes **1.16.11** per-product rail (OVERRULE 1.16.9 nest + #1611 delete).

## Remaining

- Shrink N/A on trunk (already deleted). Do not resurrect under `oya/`.
- Hub retargets on tip-free `integ/specs` only.
- PARKED — no merge to `dev` until wave review.

- `AUDIT-FINDINGS-*.json` excluded (dump-class; not forever face).
