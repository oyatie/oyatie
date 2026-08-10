# app/quality-management reorg drain notes (`integ/quality-management`)

## Ownership (rule 3d / 3e)

- **Forever home:** `app/quality-management/**` (this rail).
- **Source (read-only):** `oya/quality-management/**` on `origin/dev` until shrink-only delete on `integ/oya`.
- **Writes:** only under `app/quality-management/**` on this tip.
- **OVERRULE 3d:** product rail owns `app/quality-management/**` — never dump onto `integ/app`.

## Completed

- Wave-1 absorb: copied `oya/quality-management/**` → `app/quality-management/**` (127 files) from `origin/dev`.
- In-tree cites retargeted `oya/quality-management` → `app/quality-management` and `//oya/quality-management` → `//app/quality-management`.

- **Deepen hygiene (2026-08-10):** rewritten product-local `microservices/quality-management/` → `app/quality-management/` path cites inside forever home (hub `specs/microservices/**` + cross-product microservices cites left intact). Dest-verify COMPLETE [f60e93b2]; shrink gate ALLOWED. PARKED — no merge.

## Remaining

1. Verify destination tip contains forever bytes (this tip).
2. Shrink-only burn of `oya/quality-management/**` on `integ/oya` (NOT this rail) after verify.
3. Hub retargets (`specs/**`, capability-registry) on tip-free `integ/specs`.

## Out of envelope

- `oya/quality-management/**` deletes — `integ/oya` shrink-only only.
- `Cargo.lock` / root workspace membership — lock tip only.
- `specs/**` hub edits — `integ/specs` only.
- Sibling products under `oya/*` or `app/*` other than `quality-management`.
- `#1661` product shrink — STOP (do not touch).
