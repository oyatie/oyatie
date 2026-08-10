# app/sheets reorg drain notes (`integ/sheets`)

## Ownership (rule 3e)

- **Forever home:** `app/sheets/**` (this rail).
- **Source (read-only):** `oya/sheets/**` on `origin/dev` until shrink-only delete lands on `integ/oya`.
- **Writes:** only under `app/sheets/**` on this tip.

## Completed

- Hygiene deepen: retargeted `oya/sheets/` + `microservices/sheets/` path cites → `app/sheets/` in README/manifest (forever-home authority). (this rail)

- Wave-1 absorb: product dump to `app/sheets/` (91 files).
- Path cites rewritten `oya/sheets` → `app/sheets` inside forever home.
- BUCK cites retargeted `//oya/sheets/` → `//app/sheets/`.
1 sheets crate retained.

## Remaining for shrink phase (`integ/oya`)

- Delete absorbed `oya/sheets/**` paths after verify (shrink-only rail).
- Hub retargets on tip-free `integ/specs`.

## Out of envelope

- `oya/sheets/**` deletes — `integ/oya` shrink-only rail only.
