# app/calendar reorg drain notes (`integ/calendar`)

## Ownership (rule 3e)

- **Forever home:** `app/calendar/**` (this rail).
- **Source (read-only):** `oya/calendar/**` on `origin/dev` until shrink-only delete lands on `integ/oya`.
- **Writes:** only under `app/calendar/**` on this tip.

## Completed (this rail)

- Wave-1 absorb: product scaffold reclaim `oya/calendar/**` → `app/calendar/**` (86 files + REORG-DRAIN).
- Path cites rewritten `oya/calendar` → `app/calendar` inside forever home.
- Judgment: envelopes **1.16.8** closes land/shrink relay after **1.16.7** OVERRULE (`delete_permanently` → `reorg_now`).

## Remaining for shrink phase (`integ/oya`)

- Delete absorbed `oya/calendar/**` paths after verify (shrink-only rail).
- Do **not** undelete on `integ/oya` first — destination absorb precedes shrink.
- Hub retargets on tip-free `integ/specs`.

## Out of envelope

- `oya/calendar/**` deletes — `integ/oya` shrink-only rail only.
- `oya/governance/**` stays `delete_permanently` (META collision) — not this rail.
