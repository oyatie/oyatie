# app/social reorg drain notes (`integ/social`)

## Ownership (rule 3e)

- **Forever home:** `app/social/**` (this rail) — SEPARATE app product per ADR-0615 (not a healthcare context).
- **Source (historical):** `oya/social/**` at pre-delete SHA `4aa692919b12` (#1611 merge parent).
- **Crates:** live under `oya/community` (`oya-community-social-*`); this rail owns product kit faces only.
- **Writes:** only under `app/social/**` on this tip.

## Completed (this rail)

- Reclaim wave-1: product scaffold (catalog/contracts/slos/iac/cedar/capabilities/…) from #1611 delete.
- `AUDIT-FINDINGS-*.json` excluded.

## Remaining

- Durable-shape path-cite rewrite; hub retargets on tip-free `integ/specs`.
- Coordinate community crate rehome separately (not this PR).

## Out of envelope

- Do not resurrect under `oya/social/`.
- PARKED — no merge to `dev` until programme admits the rail.
