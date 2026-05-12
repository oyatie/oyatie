# Migration runbook template (per migrating tenant)

> Per [`docs/STANDARDS-AND-TEMPLATES.md`](../STANDARDS-AND-TEMPLATES.md) §2. Lives at `runbooks/migration/<tenant-id>-<source>-to-oyatie.md`. Validated by `runbook-discoverability`.

## Migration metadata
- **Tenant:** <tenant-id>
- **Source stack:** <vendor + version>
- **Target stack:** Oyatie <product> @ <version>
- **Migration window:** <start> .. <end>
- **Cutover dry-run dates:** <list>
- **Rollback decision deadline:** <date + UTC>

## Mapping table

| Source artifact | Target artifact | Mapping rule | Caveats |
|---|---|---|---|
| <artifact> | <artifact> | <transform> | <notes> |

## Cutover steps

1. Notify tenant operator + tenant users (T-7d, T-1d, T-0)
2. Freeze writes on source
3. Run final delta-sync
4. Verify per-class data integrity (PHI / PCI / PII / SENSITIVE_PIPA_ART23 per [ADR-0008](../decisions/ADR-0008-data-use-boundary.md))
5. Switch DNS / API endpoints
6. Resume writes on target
7. Per-tenant smoke tests
8. Per-vertical end-to-end test
9. Audit-chain emission: `EVT-TENANT-MIGRATION-CUTOVER`
10. Stop-bleeding rollback path (see §"Rollback")

## Rollback

If acceptance gates fail within first 4 hours: cut DNS back, freeze target writes, re-establish source, audit-chain `EVT-TENANT-MIGRATION-ROLLBACK`. Beyond 4h: forward-fix only (no full rollback).

## Validation gates
- ☐ Per-class data integrity (compare row counts + sample diffs)
- ☐ Per-tenant SLO recovery
- ☐ Per-vertical regulatory-evidence regen
- ☐ Tenant operator sign-off
