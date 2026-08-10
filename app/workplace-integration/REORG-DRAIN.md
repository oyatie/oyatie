# app/workplace-integration reorg drain notes (`integ/workplace-integration`)

## Ownership (rule 3d / 3e)

- **Forever home:** `app/workplace-integration/**` (this rail).
- **Source (read-only):** `oya/workplace-integration/**` on `origin/dev` until shrink-only delete on `integ/oya`.
- **Writes:** only under `app/workplace-integration/**` on this tip.
- **OVERRULE 3d:** product rail owns `app/workplace-integration/**` — never dump onto `integ/app`.

## Completed

- Wave-1 absorb: copied `oya/workplace-integration/**` → `app/workplace-integration/**` (89 files) from `origin/dev`.
- In-tree cites retargeted `oya/workplace-integration` → `app/workplace-integration` and `//oya/workplace-integration` → `//app/workplace-integration`.

## Remaining

1. Verify destination tip contains forever bytes (this tip).
2. Shrink-only burn of `oya/workplace-integration/**` on `integ/oya` (NOT this rail) after verify.
3. Hub retargets (`specs/**`, capability-registry) on tip-free `integ/specs`.

## Out of envelope

- `oya/workplace-integration/**` deletes — `integ/oya` shrink-only only.
- `Cargo.lock` / root workspace membership — lock tip only.
- `specs/**` hub edits — `integ/specs` only.
- Sibling products under `oya/*` or `app/*` other than `workplace-integration`.
- `#1661` product shrink — STOP (do not touch).
