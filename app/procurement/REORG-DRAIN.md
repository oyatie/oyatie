# app/procurement reorg drain notes (`integ/procurement`)

## Ownership (rule 3e)

- **Holding / candidate forever home:** `app/procurement/**` (this rail).
- **Source (read-only):** `oya/crm/crates/oya-procurement-source-to-pay-domain` at PRE tip `2a3dc1ebb^` (misplaced under CRM; CRM tip `integ/crm` evicted it from `app/crm/**`).
- **Writes:** only under `app/procurement/**` on this tip.

## Completed (this rail)

- **Eviction reclaim:** absorb `oya-procurement-source-to-pay-domain` (2 `.rs`) from PRE tip into `app/procurement/crates/` — not under `app/crm/**`.
- Cargo path depth to `libs/oya-data-boundary-kernel` unchanged (`../../../../libs/...`).
- BUCK deps remain `//libs/oya-data-boundary-kernel:...` (no `//oya/crm` cites).
- **Standalone nested-workspace manifest (PR #1672):** destination `Cargo.toml` states concrete
  `edition`/`version`/`rust-version` + lint baseline and declares its own `[workspace]` root, so
  the parked crate stays addressable (`cargo test --manifest-path ...` from anywhere, `cargo
  test -p` from inside the dir) while excluded from the root workspace — cargo forbids two
  workspace members with the same package name until the drain.

## Inventory (absorbed crates)

| Dir under `app/procurement/crates/` | Face | `.rs` |
|-------------------------------------|------|------:|
| `oya-procurement-source-to-pay-domain` | domain | 2 |

## Elevate (out of envelope — specs sole writer)

1. **`integ/specs` — add `roots.procurement`** — LANDED on dev: envelopes tip declares
   `roots.procurement` → `app/procurement/**` (specs/integ-branch-envelopes.json).
2. **Hub retarget** — LANDED on this rail (PR #1672): `specs/microservices/procurement.json`
   `primary_crate` / `primary_crate_ref` retargeted to `app/procurement/crates/...`, and the
   acceptance `test_ref`s run against the standalone nested-workspace manifest via
   `--manifest-path` (see Completed).
3. **Capability registry** — LANDED on this rail (PR #1672): `app/procurement` registered in
   `membership_lint_coverage.app_products.current_dirs` (governance/capability-registry.json).
4. **`integ/oya` / CRM source** — shrink-only delete leftover
   `oya/crm/.../oya-procurement-source-to-pay-domain` after verify (CRM product already drained
   separately; do not re-home under CRM).

## Out of envelope (do not touch from `integ/procurement`)

- `specs/**` hub / envelope edits — `integ/specs` only.
- `app/crm/**` — `integ/crm` only.
- `oya/crm/**` deletes — `integ/oya` shrink-only rail only.
- Other products under `oya/*` or `app/*`.

## Judgment note

Envelope CRM pragmatism already named `app/procurement/` as eviction destination; marketplace was considered and rejected (source-to-pay ≠ marketplace). Holding under `app/procurement/**` pending formal `roots.procurement` on envelopes tip.
