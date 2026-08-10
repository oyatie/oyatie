# app/real-estate reorg drain notes (`integ/real-estate`)

## Ownership (rule 3d / 3e)

- **Forever home:** `app/real-estate/**` (this rail).
- **Source (read-only):** `oya/real-estate/**` on `origin/dev` until shrink-only delete on `integ/oya`.
- **Writes:** only under `app/real-estate/**` on this tip.
- **OVERRULE 3d:** product rail owns `app/real-estate/**` — never dump onto `integ/app`.

## Completed

- Wave-1 absorb: copied `oya/real-estate/**` → `app/real-estate/**` (127 files) from `origin/dev`.
- In-tree cites retargeted `oya/real-estate` → `app/real-estate` and `//oya/real-estate` → `//app/real-estate`.

- **Deepen hygiene (2026-08-10):** rewritten product-local `microservices/real-estate/` → `app/real-estate/` path cites inside forever home (hub `specs/microservices/**` + cross-product microservices cites left intact). Dest-verify COMPLETE [f60e93b2]; shrink gate ALLOWED. PARKED — no merge.

## Remaining

1. Verify destination tip contains forever bytes (this tip).
2. Shrink-only burn of `oya/real-estate/**` on `integ/oya` (NOT this rail) after verify.
3. Hub retargets (`specs/**`, capability-registry) on tip-free `integ/specs`.

## Out of envelope

- `oya/real-estate/**` deletes — `integ/oya` shrink-only only.
- `Cargo.lock` / root workspace membership — lock tip only.
- `specs/**` hub edits — `integ/specs` only.
- Sibling products under `oya/*` or `app/*` other than `real-estate`.
- `#1661` product shrink — STOP (do not touch).
