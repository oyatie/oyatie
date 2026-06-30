## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis substrate vocabulary/dependency posture was replaced with Valkey for the ontology service in this slice:
- `specs/microservices/ontology.json` now describes instance reads as using in-memory + Valkey hot cache rather than Redis.
- `oya/ontology/manifest.json` now pins Valkey 8.1, cites ADR-0336, and declares `valkey` in `consumes_upstream_oss`.

Topology rationale:
- Ontology remains authoritative in Postgres/Citus plus ClickHouse/Iceberg history and audit-chain projections; Valkey is only a per-cell hot-cache/read-acceleration layer for rebuildable type/function reads.
- The manifest keeps three Valkey connections per tenant because query/projection load is the scaling driver and cache state can be regenerated from authoritative stores after failover.
- Valkey is intentionally omitted from `dr.backup_substrate` because the cache is derived, not a source of record.

Counterpart-fact preservations:
- None in this ontology slice; no quote-bound third-party Redis counterpart fact was present under the touched service surfaces.

Files renamed (git mv):
- None.
