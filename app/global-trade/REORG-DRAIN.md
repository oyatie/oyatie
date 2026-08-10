# app/global-trade reorg drain notes (`integ/global-trade`)

## Ownership (rule 3d / 3e)

- **Forever home:** `app/global-trade/**` (this rail).
- **Source (read-only):** `oya/global-trade/**` on `origin/dev` until shrink-only delete on `integ/oya`.
- **Writes:** only under `app/global-trade/**` on this tip.
- **OVERRULE 3d:** product rail owns `app/global-trade/**` — never dump onto `integ/app`.

## Completed

- Wave-1 absorb: copied `oya/global-trade/**` → `app/global-trade/**` (115 files) from `origin/dev`.
- In-tree cites retargeted `oya/global-trade` → `app/global-trade` and `//oya/global-trade` → `//app/global-trade`.

## Remaining

1. Verify destination tip contains forever bytes (this tip).
2. Shrink-only burn of `oya/global-trade/**` on `integ/oya` (NOT this rail) after verify.
3. Hub retargets (`specs/**`, capability-registry) on tip-free `integ/specs`.

## Out of envelope

- `oya/global-trade/**` deletes — `integ/oya` shrink-only only.
- `Cargo.lock` / root workspace membership — lock tip only.
- `specs/**` hub edits — `integ/specs` only.
- Sibling products under `oya/*` or `app/*` other than `global-trade`.
- `#1661` product shrink — STOP (do not touch).
