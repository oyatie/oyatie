# app/itsm reorg drain notes (`integ/itsm`)

## Ownership (rule 3e)

- **Forever home:** `app/itsm/**` (this rail).
- **Source (read-only):** `oya/itsm/**` on `origin/dev` until shrink-only delete lands on `integ/oya`.
- **Writes:** only under `app/itsm/**` on this tip.

## Completed

- Hygiene deepen: retargeted `oya/itsm/` + `microservices/itsm/` path cites → `app/itsm/` in README/manifest (forever-home authority). (this rail)

- Wave-1 absorb: product dump + 6 ITSM crates to `app/itsm/`.
- Path cites rewritten `oya/itsm` → `app/itsm` inside forever home.
- BUCK cites retargeted `//oya/itsm/` → `//app/itsm/`.

## Remaining for shrink phase (`integ/oya`)

- Delete absorbed `oya/itsm/**` paths after verify (shrink-only rail).
- Hub retargets on tip-free `integ/specs`.

## Out of envelope

- `oya/itsm/**` deletes — `integ/oya` shrink-only rail only.
