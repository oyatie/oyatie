# app/contract-lifecycle-management reorg drain notes (`integ/contract-lifecycle-management`)

## Ownership (rule 3d / 3e)

- **Forever home:** `app/contract-lifecycle-management/**` (this rail).
- **Source (read-only):** `oya/contract-lifecycle-management/**` on `origin/dev` until shrink-only delete on `integ/oya`.
- **Writes:** only under `app/contract-lifecycle-management/**` on this tip.
- **OVERRULE 3d:** product rail owns `app/contract-lifecycle-management/**` — never dump onto `integ/app`.

## Completed

- Wave-1 absorb: copied `oya/contract-lifecycle-management/**` → `app/contract-lifecycle-management/**` (110 files) from `origin/dev`.
- In-tree cites retargeted `oya/contract-lifecycle-management` → `app/contract-lifecycle-management` and `//oya/contract-lifecycle-management` → `//app/contract-lifecycle-management`.

## Remaining

1. Verify destination tip contains forever bytes (this tip).
2. Shrink-only burn of `oya/contract-lifecycle-management/**` on `integ/oya` (NOT this rail) after verify.
3. Hub retargets (`specs/**`, capability-registry) on tip-free `integ/specs`.

## Out of envelope

- `oya/contract-lifecycle-management/**` deletes — `integ/oya` shrink-only only.
- `Cargo.lock` / root workspace membership — lock tip only.
- `specs/**` hub edits — `integ/specs` only.
- Sibling products under `oya/*` or `app/*` other than `contract-lifecycle-management`.
- `#1661` product shrink — STOP (do not touch).
