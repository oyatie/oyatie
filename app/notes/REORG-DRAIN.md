# app/notes reorg drain notes (`integ/notes`)

## Ownership (rule 3d / 3e)

- **Forever home:** `app/notes/**` (this rail).
- **Source (read-only):** `oya/notes/**` on `origin/dev` until shrink-only delete on `integ/oya`.
- **Writes:** only under `app/notes/**` on this tip.
- **OVERRULE 3d:** product rail owns `app/notes/**` — never dump onto `integ/app`.

## Completed

- Hygiene deepen: retargeted `oya/notes/` + `microservices/notes/` path cites → `app/notes/` in README/manifest (forever-home authority).

- Slice 1: product metadata absorb — `manifest.json`, `README.md`, `slos/**`.
- Wave-1 full absorb: copied remaining `oya/notes/**` → `app/notes/**` (107 files, 1 `.rs`) from `origin/dev`.
- In-tree cites retargeted `oya/notes` → `app/notes` and `//oya/notes` → `//app/notes`.

## Remaining

1. Verify destination tip contains forever bytes (this tip).
2. Shrink-only burn of `oya/notes/**` on `integ/oya` (NOT this rail) after verify — **STOP #1661** until ordered.
3. Hub retargets (`specs/**`, capability-registry) on tip-free `integ/specs`.

## Out of envelope

- `oya/notes/**` deletes — `integ/oya` shrink-only only.
- `Cargo.lock` / root workspace membership — lock tip only.
- `specs/**` hub edits — `integ/specs` only.
- Sibling products under `oya/*` or `app/*` other than `notes`.
- `#1661` product shrink — STOP (do not touch).
