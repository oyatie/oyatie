# app/supply-chain-planning reorg drain notes (`integ/supply-chain-planning`)

## Ownership (rule 3d / 3e)

- **Forever home:** `app/supply-chain-planning/**` (this rail).
- **Source (read-only):** `oya/supply-chain-planning/**` on `origin/dev` until shrink-only delete on `integ/oya`.
- **Writes:** only under `app/supply-chain-planning/**` on this tip.
- **OVERRULE 3d:** product rail owns `app/supply-chain-planning/**` — never dump onto `integ/app`.

## Completed

- Wave-1 absorb: copied `oya/supply-chain-planning/**` → `app/supply-chain-planning/**` (127 files) from `origin/dev`.
- In-tree cites retargeted `oya/supply-chain-planning` → `app/supply-chain-planning` and `//oya/supply-chain-planning` → `//app/supply-chain-planning`.

## Remaining

1. Verify destination tip contains forever bytes (this tip).
2. Shrink-only burn of `oya/supply-chain-planning/**` on `integ/oya` (NOT this rail) after verify.
3. Hub retargets (`specs/**`, capability-registry) on tip-free `integ/specs`.

## Out of envelope

- `oya/supply-chain-planning/**` deletes — `integ/oya` shrink-only only.
- `Cargo.lock` / root workspace membership — lock tip only.
- `specs/**` hub edits — `integ/specs` only.
- Sibling products under `oya/*` or `app/*` other than `supply-chain-planning`.
- `#1661` product shrink — STOP (do not touch).
