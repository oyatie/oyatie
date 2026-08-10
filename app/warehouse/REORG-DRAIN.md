# app/warehouse reorg drain notes (`integ/warehouse`)

## Ownership (rule 3d / 3e)

- **Forever home:** `app/warehouse/**` (this rail).
- **Source (read-only):** `app/warehouse/**` on `origin/dev` until shrink-only delete on `integ/oya`.
- **Writes:** only under `app/warehouse/**` on this tip.
- **OVERRULE 3d:** product rail owns `app/warehouse/**` — never dump onto `integ/app`.

## Completed

- Wave-1 absorb: copied `app/warehouse/**` → `app/warehouse/**` (127 files) from `origin/dev`.
- In-tree cites retargeted `app/warehouse` → `app/warehouse` and `//app/warehouse` → `//app/warehouse`.

## Remaining

1. Verify destination tip contains forever bytes (this tip).
2. Shrink-only burn of `app/warehouse/**` on `integ/oya` (NOT this rail) after verify.
3. Hub retargets (`specs/**`, capability-registry) on tip-free `integ/specs`.

## Out of envelope

- `app/warehouse/**` deletes — `integ/oya` shrink-only only.
- `Cargo.lock` / root workspace membership — lock tip only.
- `specs/**` hub edits — `integ/specs` only.
- Sibling products under `oya/*` or `app/*` other than `warehouse`.
- `#1661` product shrink — STOP (do not touch).
