# Wave-2 rewrite receipt — app/payments durable shape

| Field | Value |
|-------|-------|
| judgment | envelopes `oya/payments/` → `app/payments/`, `redesign=rewrite`, `land_status=ready_for_integ_payments` |
| lane | `integ/payments` envelope `app/payments/**` |
| prior | Wave-1 absorb complete @ `d90f9f0ba` (file copy + `oya/payments`→`app/payments` cites) |
| this slice | Durable-shape rewrite of stale `microservices/payments/` product-home cites + phantom crate fiction |

## Landed shape

1. Path cites: `microservices/payments/` → `app/payments/` across forever-home product docs (capabilities, IP journeys, IPs, runbooks, PRD/README, manifest, iac, dashboards).
2. Hub refs preserved: `/specs/microservices/payments.json` unchanged (out-of-envelope hub surface).
3. Catalog: all 13 `app/payments/catalog/oya-payments-*.yaml` → `lifecycle.state=deprecated` (`deprecated_since=2026-08-10`) — zero-crate truth (#1451).
4. Manifest: `compile_surface.status=zero_crate`; bounded_contexts `crate_status=historical_inventory_only_deleted_1451`.
5. AUDIT still excluded (delete_permanently on shrink/land; not copied).

## Elevate (out of envelope)

1. **integ/oya** — shrink-only delete drained `oya/payments/**`.
2. **integ/specs** — hub retarget `governance/capability-registry.json` app_products `oya/payments` → `app/payments`.
