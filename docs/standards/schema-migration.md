---
purpose: Oyatie — Schema Migration Standard
---

# Oyatie — Schema Migration Standard

> **Owner:** `platform-eventing-og` + `axis-cloud` (per-store).
> **Companion:** ADR-0006 (Object Graph property-tier), ADR-0011 (cross-axis contract registry), ADR-0037 (public API stability tiers), [QA-TEST-STRATEGY.md](../QA-TEST-STRATEGY.md).

## 1. Universal rules

- **Versioned**: every schema carries `schema_version` field
- **Reversible**: every migration ships up + down; rollback testable
- **Dry-run**: every migration has a dry-run mode (count affected rows, log changes, no commit)
- **Backward-read** for ≥ 2 prior versions (so concurrent old + new can coexist during deploy)
- **Forward-write** for ≥ 1 next version (so newer code can write data older code can still read)
- **Per-tenant per-cell**: migration runs per-cell with per-cell rollback if it fails

## 2. Per-store schema migration

| Store | Migration tool | Versioning |
|---|---|---|
| Postgres + Citus per ADR-0045 | `sqlx migrate` (Rust-native; per-tenant shard runner) | per-table semver in migration file path |
| Object storage per ADR-0028 | object-tag versioning + lifecycle policy migration | per-bucket version |
| Search index per ADR-0030 | per-tenant index-version + reindex job | per-index semver |
| Vector index per ADR-0046 | per-tenant index version | per-collection semver |
| Event topic per ADR-0005 | CloudEvents 1.0 + Protobuf schema-registry | per-topic semver |
| Audit chain per ADR-0003 | append-only; never schema-mutate; new event class added | per-event-class semver |

## 3. Per-PR migration checklist

1. ☐ Migration file authored under `migrations/<store>/<NNNN-slug>.{up,down}.sql` (or proto / yaml)
2. ☐ Schema version bumped per [ADR-0037 public API stability](../decisions/ADR-0037-public-api-stability-tiers-and-deprecation.md) semver rules
3. ☐ Dry-run tested (per-tenant + per-cell)
4. ☐ Backward-read for prior 2 versions verified (test fixtures)
5. ☐ Forward-write for next version verified
6. ☐ Per-tenant rollback tested
7. ☐ Per-cell rollback tested (per ADR-0009)
8. ☐ Migration ledger row added (per ADR-0015 flat-crates plan §8 ledger pattern, generalized)
9. ☐ DSR cascade impact verified (does the new schema preserve DSR-cascade purge of old data?)
10. ☐ Audit-chain emission for migration steps

## 4. Anti-patterns

- Drop a column without a backward-read window — never
- Rename a field without an alias period — never (use `pub use` / view-style alias for ≥ 2 versions)
- Force-migrate all tenants in a single deploy — never (per-cell phased)
- Skip dry-run "for speed" — never
- Schema migration without rollback path — never
- Schema migration that breaks audit-chain replay — never (per ADR-0003)

## 5. Cross-axis contract migration

For schemas that span axes (per ADR-0011 cross-axis contract registry):
- Author the migration via PR touching `contracts/`
- Cross-axis review label auto-applied per [DESIGN §3.0.5.3](../DESIGN.md)
- Each affected consumer-axis team approves
- Per-PR semver-diff gate per ADR-0037
- 12-month deprecation horizon for `GA` tier; 6-month for `stable`; no horizon for `preview`

## 6. Sources
ADR-0003/0005/0006/0009/0011/0015/0028/0030/0037/0045/0046, [QA-TEST-STRATEGY.md](../QA-TEST-STRATEGY.md), [RELEASE-MANAGEMENT.md](../RELEASE-MANAGEMENT.md).
