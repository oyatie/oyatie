# app/procurement reorg drain notes (`integ/procurement`)

## Ownership (rule 3e)

- **Holding / candidate forever home:** `app/procurement/**` (this rail).
- **Source (read-only):** `oya/crm/crates/oya-procurement-source-to-pay-domain` at PRE tip `2a3dc1ebb^` (misplaced under CRM; CRM tip `integ/crm` evicted it from `app/crm/**`).
- **Writes:** only under `app/procurement/**` on this tip.

## Completed (this rail)

- **Eviction reclaim:** absorb `oya-procurement-source-to-pay-domain` (2 `.rs`) from PRE tip into `app/procurement/crates/` — not under `app/crm/**`.
- Cargo path depth to `libs/oya-data-boundary-kernel` unchanged (`../../../../libs/...`).
- BUCK deps remain `//libs/oya-data-boundary-kernel:...` (no `//oya/crm` cites).

## Inventory (absorbed crates)

| Dir under `app/procurement/crates/` | Face | `.rs` |
|-------------------------------------|------|------:|
| `oya-procurement-source-to-pay-domain` | domain | 2 |

## Elevate (out of envelope — specs sole writer)

1. **`integ/specs` — add `roots.procurement`** — no envelope root exists today (`has procurement? False` on envelopes tip). Forward-declare `integ/procurement` → `app/procurement/**` (or judged capability path). This rail is provisional until that land.
2. **Hub retarget** — `specs/microservices/procurement.json` still cites `primary_crate: microservices/procurement/crates/oya-procurement-source-to-pay-domain`; retarget to `app/procurement/crates/...` (and test_ref paths) on tip-free `integ/specs`.
3. **Capability registry** — register/confirm procurement product forever home if not already an `app_products` entry.
4. **`integ/oya` / CRM source** — shrink-only delete leftover `oya/crm/.../oya-procurement-source-to-pay-domain` after verify (CRM product already drained separately; do not re-home under CRM).

## Out of envelope (do not touch from `integ/procurement`)

- `specs/**` hub / envelope edits — `integ/specs` only.
- `app/crm/**` — `integ/crm` only.
- `oya/crm/**` deletes — `integ/oya` shrink-only rail only.
- Other products under `oya/*` or `app/*`.

## Judgment note

Envelope CRM pragmatism already named `app/procurement/` as eviction destination; marketplace was considered and rejected (source-to-pay ≠ marketplace). Holding under `app/procurement/**` pending formal `roots.procurement` on envelopes tip.
