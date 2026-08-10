# app/payroll reorg drain notes (`integ/payroll`)

## Ownership (rule 3d / 3e)

- **Forever home:** `app/payroll/**` (this rail).
- **Source (read-only):** `oya/payroll/**` on `origin/dev` until shrink-only delete on `integ/oya`.
- **Writes:** only under `app/payroll/**` on this tip.
- **OVERRULE 3d:** product rail owns `app/payroll/**` — never dump onto `integ/app`.

## Completed

- Wave-1 absorb: copied `oya/payroll/**` → `app/payroll/**` (39 files) from `origin/dev`.
- In-tree cites retargeted `oya/payroll` → `app/payroll` and `//oya/payroll` → `//app/payroll`.

## Remaining

1. Verify destination tip contains forever bytes (this tip).
2. Shrink-only burn of `oya/payroll/**` on `integ/oya` (NOT this rail) after verify.
3. Hub retargets (`specs/**`, capability-registry) on tip-free `integ/specs`.

## Out of envelope

- `oya/payroll/**` deletes — `integ/oya` shrink-only only.
- `Cargo.lock` / root workspace membership — lock tip only.
- `specs/**` hub edits — `integ/specs` only.
- Sibling products under `oya/*` or `app/*` other than `payroll`.
- `#1661` product shrink — STOP (do not touch).
