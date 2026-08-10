# app/docs reorg drain notes (`integ/app-docs`)

## Ownership (rule 3d / 3e)

- **Forever home:** `app/docs/**` (this rail — product docs app).
- **Branch naming:** `integ/app-docs` (avoids collision with docs *plane* `integ/docs` which owns `docs/**` + `templates/**`).
- **Source (read-only):** `oya/docs/**` on `origin/dev` until shrink-only delete lands on `integ/oya`.
- **Writes:** only under `app/docs/**` on this tip.
- **OVERRULE 3d:** migrated off shared `integ/app` (wrong multi-product absorb shape).

## Completed (this rail)

- Wave-1 absorb: 89 files to forever home (replayed from `integ/app@f7133b24b` / prior `63f64327b`).

## Remaining for shrink phase (`integ/oya`)

- Delete absorbed `oya/docs/**` paths after verify (shrink-only rail).
- Hub retargets on tip-free `integ/specs`.

## Out of envelope (do not touch from `integ/app-docs`)

- `oya/docs/**` deletes — `integ/oya` shrink-only rail only.
- Docs *plane* paths (`docs/**`, `templates/**`) — `integ/docs` only.
- Other products under `oya/*` or `app/*`.
