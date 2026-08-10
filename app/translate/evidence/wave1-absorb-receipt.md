# Wave-1 absorb receipt — oya/translate → app/translate

| Field | Value |
|-------|-------|
| judgment | envelopes `8bcb65d08` — `oya/translate/` → `app/translate/`, `destination_integ=integ/translate`, `land_status=ready_for_integ_app`, `redesign=rewrite` |
| lane | `integ/translate` envelope forever path `app/translate/**` |
| source | `oya/translate/**` (left in place on origin/dev) |
| forever | `app/translate/**` (85 files absorbed) |

## Landed shape

Product dump rewrite into `app/translate/`. BUCK cites retargeted.

## Elevate (out of envelope)

1. **integ/oya** — delete drained `oya/translate/**` after verify.
2. **integ/specs** — hub retarget follow-on.
