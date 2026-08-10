# Wave-1 absorb receipt — oya/payments → app/payments

| Field | Value |
|-------|-------|
| judgment | `#1644@0c6284cdef` envelopes 1.16.0 — `oya/payments/` → `app/payments/`, `land_status=ready_for_integ_payments`, `redesign=rewrite` |
| lane | `integ/payments` envelope `app/payments/**` |
| source | `oya/payments/**` (102 files; left in place; delete owned by `integ/oya`) |
| forever | `app/payments/**` (96 files absorbed; AUDIT excluded) |

## Landed shape

Zero-crate product dump rewrite into `app/payments/` (NOT `billing/`). Slices 1–2 complete.

## Elevate (out of envelope)

1. **integ/oya** — delete drained `oya/payments/**` after this absorb receipt.
2. **integ/specs** — hub retarget `specs/capability-registry.json` app_products `oya/payments` → `app/payments`.
