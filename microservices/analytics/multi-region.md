# Analytics µservice — Multi-Region

**Authority:** ADR-0049 cross-region replication and residency, ADR-0010 regional packs, ADR-0009 cell architecture, ADR-0193
**Last reviewed:** 2026-05-18

## Residency model (per `oya-tenancy-kernel::ResidencyClass`)

| Class | Primary region | Failover region | Cross-region data egress |
|---|---|---|---|
| StrictKR | kr-* cells only | none | FORBIDDEN |
| StrictEU | eu-* cells only | none | FORBIDDEN |
| KrWithUsFailover | kr-* primary | us-* (DR only) | Permitted on explicit DR scenario |
| Global | any | per-cell strategy | Permitted |

## Per-cell deployment shape

Each cell hosts:

- 1 ClickHouse cluster (3 shards × 2 replicas + 3-node Keeper) — analytics-namespace.
- Per-tenant databases for tenants whose `ResidencyClass` permits this cell.
- Cell-local S3-compat (SeaweedFS) for cold tier.
- Cell-local OpenBao for credentials.

## Cross-cell aggregation (rare, internal only)

Cross-cell `remote()` queries are forbidden for tenant principals; permitted only for `Role::InternalAdmin`. Pattern: per-cell rollups → scheduled cross-cell aggregation job emits a fleet-wide rollup table at midnight UTC, read by internal ops dashboards.

## DR shape (per ADR-0180-dr-business-continuity-portfolio-policy)

- **RPO:** ≤ 24h (matches ADR-0152). Hot tier replicated within-cell every 60s; cold tier replicated cell-locally to S3.
- **RTO:** ≤ 1h per affected tenant.
- **Cross-cell failover:** Only KrWithUsFailover + Global tenants. Failover triggers documented in `microservices/analytics/runbooks/incident-response.md` (deferred).
- **Backup cross-cell replication:** Daily backup → secondary cell within the same residency boundary.

## Pack overlays

- `microservices/analytics/iac/kustomize/overlays/pack-kr/` — Korean pack; S3 endpoint kr-* only; node selector `oya.pack=kr`.
- `microservices/analytics/iac/kustomize/overlays/pack-eu/` — European pack; S3 endpoint eu-* only; node selector `oya.pack=eu`.

NetworkPolicy + Cedar enforce cross-pack denial (per ADR-0049 + ADR-0010).

## References

- ADR-0049, ADR-0010, ADR-0009, ADR-0152, ADR-0180-dr-business-continuity-portfolio-policy, ADR-0193.
