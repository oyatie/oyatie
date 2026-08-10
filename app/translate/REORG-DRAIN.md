# app/translate reorg drain notes (`integ/translate`)

## Ownership (rule 3d / 3e)

- **Forever home:** `app/translate/**` (this rail).
- **Source (read-only):** `oya/translate/**` on `origin/dev` until shrink-only delete lands on `integ/oya`.
- **Writes:** only under `app/translate/**` on this tip.
- **OVERRULE 3d:** migrated off shared `integ/app` (wrong multi-product absorb shape). Source tip replay: `integ/app@7e3e1f82d`.

## Completed (this rail)

- Wave-1 absorb: product tree landed in forever home (86 files; replayed from `integ/app`).
- Path cites rewritten `oya/translate` → `app/translate` where present in absorb commit.

## Remaining for shrink phase (`integ/oya`)

- Delete absorbed `oya/translate/**` paths after verify (shrink-only rail).
- Hub retargets on tip-free `integ/specs` (`destination_integ=integ/translate`).

## Out of envelope (do not touch from `integ/translate`)

- `oya/translate/**` deletes — `integ/oya` shrink-only rail only.
- Other products under `oya/*` or `app/*`.
