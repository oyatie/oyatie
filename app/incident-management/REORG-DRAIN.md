# app/incident-management reorg drain notes (`integ/incident-management`)

## Ownership (rule 3d / 3e)

- **Forever home:** `app/incident-management/**` (this rail).
- **Source (read-only):** `oya/incident-management/**` on `origin/dev` until shrink-only delete on `integ/oya`.
- **Writes:** only under `app/incident-management/**` on this tip.
- **OVERRULE 3d:** product rail owns `app/incident-management/**` — never dump onto `integ/app`.

## Completed

- Wave-1 absorb: copied `oya/incident-management/**` → `app/incident-management/**` (134 files) from `origin/dev`.
- In-tree cites retargeted `oya/incident-management` → `app/incident-management` and `//oya/incident-management` → `//app/incident-management`.

- **Deepen hygiene (2026-08-10):** rewritten product-local `microservices/incident-management/` → `app/incident-management/` path cites inside forever home (hub `specs/microservices/**` + cross-product microservices cites left intact). Dest-verify COMPLETE [f60e93b2]; shrink gate ALLOWED. PARKED — no merge.

## Remaining

1. Verify destination tip contains forever bytes (this tip).
2. Shrink-only burn of `oya/incident-management/**` on `integ/oya` (NOT this rail) after verify.
3. Hub retargets (`specs/**`, capability-registry) on tip-free `integ/specs`.

## Out of envelope

- `oya/incident-management/**` deletes — `integ/oya` shrink-only only.
- `Cargo.lock` / root workspace membership — lock tip only.
- `specs/**` hub edits — `integ/specs` only.
- Sibling products under `oya/*` or `app/*` other than `incident-management`.
- `#1661` product shrink — STOP (do not touch).
