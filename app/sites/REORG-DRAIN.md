# app/sites reorg drain notes (`integ/sites`)

## Ownership (rule 3d / 3e)

- **Forever home:** `app/sites/**` (this rail).
- **Source (read-only):** `oya/sites/**` on `origin/dev` until shrink-only delete lands on `integ/oya`.
- **Writes:** only under `app/sites/**` on this tip.
- **OVERRULE 3d:** migrated off shared `integ/app`. Replay tip `integ/app@232ef342c` (full dump).

## Completed

- Hygiene deepen: retargeted `oya/sites/` + `microservices/sites/` path cites → `app/sites/` in README/manifest (forever-home authority). (this rail)

- Wave-1 absorb: product tree in forever home (replayed from `integ/app`).

## Remaining for shrink phase (`integ/oya`)

- Delete absorbed `oya/sites/**` after verify.
- Hub retarget `destination_integ=integ/sites` on `integ/specs`.

## Out of envelope

- `oya/sites/**` deletes — `integ/oya` only.
- Other `app/*` products.
