# Wave-1 absorb receipt — oya/itsm → app/itsm

| Field | Value |
|-------|-------|
| judgment | envelopes `2adc2f038` — `oya/itsm/` → `app/itsm/`, `land_status=ready_for_integ_itsm`, `redesign=rewrite` |
| lane | `integ/itsm` envelope `app/itsm/**` |
| source | `oya/itsm/**` (left in place on origin/dev) |
| forever | `app/itsm/**` (161 files absorbed) |

## Landed shape

Product dump + 6 ITSM crates rewrite into `app/itsm/`. BUCK cites retargeted.

## Elevate (out of envelope)

1. **integ/oya** — delete drained `oya/itsm/**` after verify.
2. **integ/specs** — hub retarget follow-on.
