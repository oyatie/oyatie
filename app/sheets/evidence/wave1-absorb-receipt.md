# Wave-1 absorb receipt — oya/sheets → app/sheets

| Field | Value |
|-------|-------|
| judgment | envelopes `2adc2f038` — `oya/sheets/` → `app/sheets/`, `land_status=ready_for_integ_sheets`, `redesign=rewrite` |
| lane | `integ/sheets` envelope `app/sheets/**` |
| source | `oya/sheets/**` (left in place on origin/dev) |
| forever | `app/sheets/**` (91 files absorbed) |

## Landed shape

Product dump rewrite into `app/sheets/`. BUCK cites retargeted.

## Elevate (out of envelope)

1. **integ/oya** — delete drained `oya/sheets/**` after verify.
2. **integ/specs** — hub retarget follow-on.
