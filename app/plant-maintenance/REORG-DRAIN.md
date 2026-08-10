# app/plant-maintenance reorg drain notes (`integ/plant-maintenance`)

## Ownership (rule 3d / 3e)

- **Forever home:** `app/plant-maintenance/**` (this rail).
- **Source (read-only):** `oya/plant-maintenance/**` on `origin/dev` until shrink-only delete on `integ/oya`.
- **Writes:** only under `app/plant-maintenance/**` on this tip.
- **OVERRULE 3d:** product rail owns `app/plant-maintenance/**` — never dump onto `integ/app`.

## Completed

- Wave-1 absorb: copied `oya/plant-maintenance/**` → `app/plant-maintenance/**` (127 files) from `origin/dev`.
- In-tree cites retargeted `oya/plant-maintenance` → `app/plant-maintenance` and `//oya/plant-maintenance` → `//app/plant-maintenance`.

## Remaining

1. Verify destination tip contains forever bytes (this tip).
2. Shrink-only burn of `oya/plant-maintenance/**` on `integ/oya` (NOT this rail) after verify.
3. Hub retargets (`specs/**`, capability-registry) on tip-free `integ/specs`.

## Out of envelope

- `oya/plant-maintenance/**` deletes — `integ/oya` shrink-only only.
- `Cargo.lock` / root workspace membership — lock tip only.
- `specs/**` hub edits — `integ/specs` only.
- Sibling products under `oya/*` or `app/*` other than `plant-maintenance`.
- `#1661` product shrink — STOP (do not touch).
