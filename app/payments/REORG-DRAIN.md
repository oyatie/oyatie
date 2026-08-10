# app/payments reorg drain notes (`integ/payments`)

## Ownership (rule 3e)

- **Forever home:** `app/payments/**` (this rail).
- **Source (read-only):** `oya/payments/**` on `origin/dev` until shrink-only delete lands on `integ/oya`.
- **Writes:** only under `app/payments/**` on this tip.

## Completed (this rail)

- Slice 1: product metadata absorb — `manifest.json`, `README.md`, `PRD.md`, `PHASE-01-PAYMENTS-MVP.md`, `slos/**`.
- Slice 2: contracts + policy + capabilities + catalog + IPs + iac + dashboards + runbooks + scorecards + security + dpia + decisions.
- Slice 3: IP journey markdown root files (6 files).
- **Wave-1 complete:** 103 files in forever home (101 source ex-AUDIT + REORG-DRAIN + evidence).
- Slice 4 / **Wave-2 rewrite:** durable-shape path cite fix — `microservices/payments/` → `app/payments/` inside forever home (hub `specs/microservices/payments.json` left intact); catalog `oya-payments-*` lifecycle → `deprecated` (phantom crate cluster deleted #1451); `manifest.compile_surface=zero_crate`.
- Path cites rewritten `oya/payments` → `app/payments` inside forever home.
- `AUDIT-FINDINGS-2026-05-20.json` excluded per judgment (delete_permanently).
- **ELEVATE #1451 CLEARED (verdict A):** intentional zero-crate product — crates stay deleted; metadata forever home OK. See `evidence/elevate-1451-judgment.md`.

## Judgment — ELEVATE #1451 payments crate loss (2026-08-10)

| Field | Value |
|-------|-------|
| Verdict | **A** — intentional zero-crate product (`keep_deleted`) |
| Rejected | **B** reclaim `oya-payments-*` onto `app/payments/crates/**` |
| Tip | this rail (`integ/payments` / PR #1663) |
| Evidence | `evidence/elevate-1451-judgment.md` |

**Binding:** do **not** restore the 20-crate cluster. Product truth is specification + policy/IaC/SLO surfaces with `compile_surface.status=zero_crate`. Reclaim would reintroduce dishonest "looks built" scaffolding (#1451 rationale). Contrast office/crm (#1664/#1665): those were shrink-before-durable-home bugs with live Rust callers — payments is not that class.

**Optional specs note:** envelopes already encode zero-crate (`oya/payments/` judgment + #1451 cite). If a specs agent wants a `keep_deleted` / `do_not_reclaim` ledger tag, bump only on tip-free `integ/specs` — this rail does not touch hubs.

## Remaining for shrink phase (`integ/oya`)

- Delete absorbed `oya/payments/**` paths after verify (shrink-only rail).
- Hub retargets (`specs/capability-registry.json` app_products) on tip-free `integ/specs`.

## Out of envelope (do not touch from `integ/payments`)

- `oya/payments/**` deletes — `integ/oya` shrink-only rail only.
- Other products under `oya/*` or `app/*`.
- Crate reclaim / `app/payments/crates/**` — **banned** under verdict A.
- `#1661` further product shrink — STOP (orchestrator hard); this rail never shrinks `integ/oya`.
