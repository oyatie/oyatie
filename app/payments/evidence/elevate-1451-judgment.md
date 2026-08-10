# ELEVATE #1451 — payments crate loss judgment

| Field | Value |
|-------|-------|
| Date | 2026-08-10 |
| Rail | `integ/payments` / PR **#1663** |
| Verdict | **A** — intentional zero-crate product; crates stay deleted; metadata OK |
| Rejected | **B** — reclaim crates from pre-#1451 tip onto `app/payments/crates/**` |
| Prior tip | `41c091763` (wave-2 already annotated `compile_surface=zero_crate`) |
| HARD | no merge; do not shrink #1661 |

## Evidence (load-bearing)

1. **#1451 MERGED** (`e25797cdf`) — founder-approved delete of closed `oya/payments` crate cluster; kept 102 specification files. Decisive probe: **external consumers of any `oya-payments-*` crate = 0** (self-referential component only). Deleting cannot break a caller because there is no caller.
2. **Not a husk misread:** crates carried types/traits (clean-architecture domain shape); deletion was for honesty (unbuilt product must not look built), not "empty fn count".
3. **Tip truth before this note:** `app/payments/**` has **0** `Cargo.toml`; `manifest.compile_surface.status=zero_crate` + `deleted_crate_cluster_pr=1451`; catalog `oya-payments-*` → `lifecycle.state=deprecated`; bounded_contexts `crate_status=historical_inventory_only_deleted_1451`.
4. **Envelopes already agree:** `specs/integ-branch-envelopes.json` `oya/payments/` → `app/payments/` with shape `zero-crate product dump rewrite` and cartesian doubt citing crate cluster deleted #1451.
5. **Peer contrast:** office/crm reclaim (#1664/#1665) cleared a shrink-before-durable-home HARD BUG with live Rust. Plan SSOT marks **payments OK** in that OVERRULE — not the same defect class. Calendar OVERRULE used payments as the correct zero-crate *metadata* peer, not a reclaim peer.
6. **Scout SHA note:** mission cited `f3362ac4`; deletion landing commit is **`e25797cdf` (#1451)**. `f3362ac4` is unrelated CI runner fleet work; do not treat it as the crate-delete tip.

## Why not B

Product does **not** need a compile surface today. Restoring 20 closed crates would recreate the fiction #1451 removed. Future implementation lands new owned crates under forever-home faces when a real IP delivers callers — not by resurrecting the deleted cluster.

## Specs / envelopes

`integ/specs` tip busy (active 1.16.x bumps). **No envelopes edit from this rail.** Optional `keep_deleted` / `do_not_reclaim` ledger tag → elevate to specs agent when tip-free. Tip-local REORG-DRAIN + this receipt are durable enough for drain ownership.

## Elevate (residual, out of envelope)

1. **integ/oya** — shrink-only delete drained `oya/payments/**` after verify (STOP further product shrink per orchestrator; do not deepen #1661 beyond verified absorbs).
2. **integ/specs** — hub retarget `specs/capability-registry.json` app_products `oya/payments` → `app/payments`; optional judgment ledger tag `crate_disposition=keep_deleted_1451` when tip-free.
