# app/slides reorg drain notes (`integ/slides`)

## Ownership (rule 3d / 3e)

- **Forever home:** `app/slides/**` (this rail).
- **Source (read-only):** `oya/slides/**` on `origin/dev` until shrink-only delete on `integ/oya`.
- **Writes:** only under `app/slides/**` on this tip.
- **OVERRULE 3d:** product rail owns `app/slides/**` — never dump onto `integ/app`.

## Completed

- Slice 1: product metadata absorb — `manifest.json`, `README.md`, `slos/**`.
- Wave-1 full absorb: copied remaining `oya/slides/**` → `app/slides/**` (93 files, 1 `.rs`) from `origin/dev`.
- In-tree cites retargeted `oya/slides` → `app/slides` and `//oya/slides` → `//app/slides`.

## Remaining

1. Verify destination tip contains forever bytes (this tip).
2. Shrink-only burn of `oya/slides/**` on `integ/oya` (NOT this rail) after verify — **STOP #1661** until ordered.
3. Hub retargets (`specs/**`, capability-registry) on tip-free `integ/specs`.

## Out of envelope

- `oya/slides/**` deletes — `integ/oya` shrink-only only.
- `Cargo.lock` / root workspace membership — lock tip only.
- `specs/**` hub edits — `integ/specs` only.
- Sibling products under `oya/*` or `app/*` other than `slides`.
- `#1661` product shrink — STOP (do not touch).
