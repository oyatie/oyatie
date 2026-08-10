# Wave-1 absorb receipt — oya/hr → app/hr

| Field | Value |
|-------|-------|
| judgment | envelopes `8bcb65d08` — `oya/hr/` → `app/hr/`, `destination_integ=integ/app`, `land_status=ready_for_integ_app`, `redesign=rewrite` |
| lane | `integ/app` envelope forever path `app/hr/**` |
| source | `oya/hr/**` (left in place on origin/dev) |
| forever | `app/hr/**` (38 files absorbed) |

## Landed shape

Product dump rewrite into `app/hr/`. BUCK cites retargeted.

## Elevate (out of envelope)

1. **integ/oya** — delete drained `oya/hr/**` after verify.
2. **integ/specs** — hub retarget follow-on.
