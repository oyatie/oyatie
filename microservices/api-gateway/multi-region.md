# api-gateway — Multi-region behaviour

**Authority:** ADR-0248 (cellular) + ADR-0252 (HLC/TrueTime) + ADR-0244 (tenant scoping).
**Hyperscaler precedent:** AWS Multi-Region Architecture + Cloudflare global Anycast network + Spanner global consistency model.

## A — Cell topology

Per ADR-0248. 11 sovereign and non-sovereign regions; 24 Tier-0 edge cells.

| Region | Cells | Sov-cell? | Audience served |
|---|---:|---|---|
| us-east-{1,2,3,4} | 4 | No (default) | NA |
| us-west-{1,2,3} | 3 | No (default) | NA |
| eu-frankfurt-{1,2,3} | 3 | EU-sov-cell variant for `pack-eu` | EU |
| eu-ireland-{1,2} | 2 | EU-sov-cell variant | EU |
| ap-tokyo-{1,2} | 2 | No | JP |
| ap-seoul-{1,2,3} | 3 | sov-cell-kr (PIPA/CSAP) | KR resident |
| ap-singapore-{1,2} | 2 | No | SEA |
| cn-shanghai-{1,2} | 2 | sov-cell-cn (PIPL) | PRC resident |
| sa-saopaulo-1 | 1 | LGPD overlay | BR |
| me-dubai-1 | 1 | sov-cell-ae (UAE PDPL) | UAE |
| me-riyadh-1 | 1 | sov-cell-ksa (KSA PDPL) | KSA |
| (US Government cells) | per-contract | sov-cell-il5/6, sov-cell-fedramp-high | US Federal |

## B — Per-tenant routing

- **Default tenants:** route to nearest cell via Anycast.
- **Pack-eu tenants:** route to nearest EU-sov-cell; cross-border to non-EU forbidden by Cedar.
- **Pack-kr / pack-cn / pack-ae / pack-ksa tenants:** route only to respective sov-cell.
- **Pack-us-healthcare tenants:** any US cell with HIPAA BAA in place.
- **Pack-il5/6 tenants:** route only to sov-cell-il5/6 (per-contract).

Routing decision is made at the BGP-Anycast layer + at the gateway's per-tenant `home_cell` + `dr_cell` map (per ADR-0244).

## C — Failover behaviour

- **Single-cell failure:** NS1 health-check de-pools the cell from Anycast in ≤60s; traffic re-routes to nearest healthy cell.
- **Multi-cell region failure:** Sov-cell tenants get 503 + Retry-After (sov-cells have no cross-region failover by design); non-sov-cell tenants re-route cross-region.
- **Sov-cell-kr failure:** `dr_cell = ap-seoul-2` (within KR territory); never failover to non-KR cell.
- **Sov-cell-cn failure:** `dr_cell = cn-shanghai-2`; never failover outside PRC.

## D — Cross-region behaviour

- **Rate-limit counters:** per-cell-local; cross-cell aggregation via Kafka tick (eventually consistent, ≤10s lag). Tenants are NOT promised cross-cell strict-consistency on rate-limit; SLA documents this.
- **Cedar fragments:** push-based from policy-engine ledger; ≤30s freshness across all cells.
- **TLS certs:** cert-manager per cell; CT logs global; per-cell key isolated.
- **ECH configs:** per-cell key; global HTTPS RR rotation coordinated.
- **Audit events:** local emission + cross-region forward to audit-chain global ledger; HLC ordering per ADR-0252.

## E — Per-cell shuffle-sharding

Per ADR-0248. Tenants are shuffle-sharded across cells in their region. Single tenant rarely shares a cell with another tenant of the same "noisy neighbour" class. Reduces blast radius.

- Shard width: 4 cells per region (for regions with ≥4 cells).
- Shard assignment: `hash(tenant_id) → 4-of-N cell subset`.
- Re-shard cadence: monthly or on cell addition/removal.

## F — Latency budget cross-region

| Path | Latency budget |
|---|---:|
| Client → nearest cell (Anycast) | ≤30ms p50 |
| Cell → upstream µservice (same cell) | ≤2ms p50 |
| Cell → upstream µservice (cross-cell same region) | ≤5ms p50 |
| Cell → upstream µservice (cross-region) | ≤80ms p50 (rare; only DR fallback) |
| Audit emission cross-region | async; not on hot path |

## G — Disaster recovery

- **RPO (recovery-point objective):** 0 for in-flight requests (audit-chain async — best-effort < 10s gap on crash). 
- **RTO (recovery-time objective):** ≤60s for cell failover; ≤5min for full-region failover; per-contract for sov-cell DR.
- **DR test cadence:** monthly per-region cell evac drill; quarterly multi-region failover drill.

## H — References

- ADR-0248, ADR-0252, ADR-0244
- `microservices/api-gateway/ARCHITECTURE.md`
- `microservices/api-gateway/runbooks/cell-evac.md`
- AWS Multi-Region Architecture whitepaper 2024
- Cloudflare Global Network technical brief 2024
