# Wave-1 absorb receipt — oya/sites → app/sites

| Field | Value |
|-------|-------|
| judgment | envelopes `8bcb65d08` — `oya/sites/` → `app/sites/`, `destination_integ=integ/sites`, `land_status=ready_for_integ_app`, `redesign=rewrite` |
| lane | `integ/sites` envelope forever path `app/sites/**` |
| source | `oya/sites/**` (left in place on origin/dev) |
| forever | `app/sites/**` (86 files absorbed) |

## Landed shape

Product dump rewrite into `app/sites/`. BUCK cites retargeted.

## Elevate (out of envelope)

1. **integ/oya** — delete drained `oya/sites/**` after verify.
2. **integ/specs** — hub retarget follow-on.
