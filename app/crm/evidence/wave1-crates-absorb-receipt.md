# Wave-1 crates absorb receipt — oya/crm crates → app/crm

| Field | Value |
|-------|-------|
| bug | scout: shrink deleted 16 `.rs` at PRE but integ/crm tip was metadata-only (0 rust) |
| PRE | `2a3dc1ebb^` = `ebecf90070aaae6213e5bf0044ef3087561dd0cc` |
| lane | `integ/crm` envelope `app/crm/**` |
| source | `oya/crm/crates/oya-crm-*` (left in place dual-home) |
| forever | `app/crm/crates/oya-crm-*` |

## Landed shape

- Absorbed **2** CRM crates / **14** `.rs` / **19** tracked crate files into `app/crm/crates/`.
- BUCK cites retargeted `//oya/crm/` → `//app/crm/`.
- `oya-procurement-source-to-pay-domain` (**2** `.rs`) **evicted** — not under `app/crm/**`.

## Elevate

1. **procurement rail** — forever home for `oya-procurement-source-to-pay-domain` (not CRM).
2. **integ/oya** — shrink-only delete drained `oya/crm/**` after verify.
3. **integ/specs** — hub / microservice path retarget follow-on.
