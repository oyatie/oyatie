# Wave-1 absorb receipt — oya/docs → app/docs

| Field | Value |
|-------|-------|
| judgment | envelopes `2adc2f038` — `oya/docs/` → `app/docs/`, `land_status=ready_for_integ_app`, `redesign=rewrite` |
| lane | `integ/app-docs` envelope `app/docs/**` |
| source | `oya/docs/**` (left in place on origin/dev) |
| forever | `app/docs/**` (89 files absorbed) |

## Landed shape

Product glue rewrite into `app/docs/`. translate/sites parked on source.

## Elevate (out of envelope)

1. **integ/oya** — delete drained `oya/docs/**` after verify.
2. **integ/app-docs** — translate/sites full absorb follow-on when un-parked.
