# Wave-1 absorb receipt — oya/office → app/office

| Field | Value |
|-------|-------|
| judgment | `#1644@0c6284cdef` envelopes 1.16.0 — `oya/office/` → `app/office/`, `land_status=ready_for_integ_office`, `redesign=rewrite` |
| lane | `integ/office` envelope `app/office/**` |
| source | `oya/office/**` (63 files; 19 crates; left in place) |
| forever | `app/office/**` (63 crate files absorbed + drain/manifest/receipt) |

## Landed shape

19-crate forest rewrite into `app/office/`. BUCK cites retargeted `//oya/office/` → `//app/office/`. Substrate port burn deferred (see `REORG-DRAIN.md`).

## Elevate (out of envelope)

1. **integ/oya** — delete drained `oya/office/**` after verify.
2. **integ/specs** — hub retarget + crate rename follow-on.
3. **capability rails** (`storage/`, `tenancy/`, `iam/`, search) — substrate-port rewrite homes.
