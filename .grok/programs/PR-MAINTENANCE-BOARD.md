# Open PR maintenance board

**Updated:** 2026-08-05  
**Policy:** dual-critic APPROVE + oya-ci-required SUCCESS → mm-drive squash (auto_undraft). Human APPROVE not mandatory.

| PR | Title | Dual-critic | CI (last) | Fix agent | Bead |
|----|-------|-------------|-----------|-----------|------|
| 1561 | k8s W0-A | APPROVE | buck2 **FAIL** (diagnose) | Fix 1561 agent | oyatie-oso.3 / 7xf |
| 1562 | R2 path-filter | APPROVE | pending | wait | oyatie-oso.2 |
| 1563 | CAS 3A | APPROVE | partial | wait | oyatie-oso.5 |
| 1564 | R1 runners | APPROVE | buck2 **FAIL** unjustified/unreachable +1 | Fix 1564 agent | oyatie-oso.1 |
| 1565 | Face de-commit | APPROVE | pending | wait | oyatie-oso.15 |
| 1566 | Harness brand | APPROVE | pending | wait | oyatie-oso.16 |

## Done this cycle
- G039 packet closed (oyatie-oso.4) trunk green 30999838837
- Parallel: move-plan hygiene, dual-home inventory, kit mm-packet commit

## Ops residual
- After #1564 green+merge: apply RUNBOOK-scale-runners.md (human)
