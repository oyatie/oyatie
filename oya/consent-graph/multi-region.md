# consent-graph multi-region architecture

- Owner: axis-consent-graph
- Date: 2026-05-18
- Authority: ADR-0214 §2.5, ADR-SVC-CG-004 (grantor-region topic ownership), ADR-0028 (cloud arch).

## 1. Topology

consent-graph runs **active-active per region** with the following per-region footprint:

- 3× `consent-graph-app` pods (agreement, enforcement, projection-gateway co-resident)
- 3× `revocation-app` pods
- 2× `partner-directory-app` pods
- 1× `consent-graph-worker` pod (background reconciliation; HPA up to 5)
- 1× `audit-bridge-app` pod (HPA up to 10 on Pulsar lag)

Regions (year-1):
- us-east-1, us-west-2 (us pack)
- eu-west-1, eu-central-1 (eu pack)
- ap-northeast-2 (kr pack)
- ap-northeast-1 (jp pack)
- ap-southeast-1 (sg pack)
- ap-southeast-2 (au pack)
- ap-south-1 (in pack)
- sa-east-1 (br pack)
- me-south-1 (ae, ksa packs)

## 2. Grantor-region authority (ADR-SVC-CG-004)

Each agreement has a *home* region — the grantor's region. The home region is the authoritative
source for:
- The `consent_graph_agreements` row.
- The compiled Cedar policy.
- The projection topic (in the grantor's local Pulsar cluster).

Reads from other regions go cross-region; writes always route to home region.

## 3. Postgres + Citus topology

Per-region Citus cluster:
- Coordinator: 3 nodes (HA)
- Workers: 16 nodes (year-1; scale on shard size)
- Distribution key: `grantor_tenant_id` (per IP-003 §4)
- Replication: synchronous across 3 AZs in region; asynchronous to one DR region

Cross-region writes: routed via "global writes" pattern — application-level region routing in
consent-graph-app's adapter chooses home-region's coordinator. Latency cost for cross-region write:
~80ms p50 us-east↔eu-west, ~150ms us-east↔ap-northeast.

## 4. Pulsar topology

### 4.1 Per-region cluster
- 3 brokers per cluster (year-1; scale on partition count)
- 3 BookKeeper bookies per cluster (replication factor 3, ack 2)
- 1 ZK ensemble (3 nodes)

### 4.2 Cross-region replication
- `oya.consent-graph.revocation.v1`: full georeplication across all regions (high-priority lane).
- `oya.consent-graph.audit-bridge.v1`: full georeplication (for cross-region audit emission).
- `oya.consent-graph.projection.v1.*`: NO automatic georeplication; per-agreement opt-in only via
  `geo_replicate_to_grantee_region` flag (sovereignty default).

### 4.3 Georeplication SLO
- Lag p99 ≤500ms for revocation topic (per IP-008 §5).
- Lag p99 ≤2s for audit-bridge topic.

## 5. Routing model

### 5.1 North-south traffic
1. Client → CloudFront/Akamai (region-aware GeoDNS).
2. → Region's edge api-gateway (Istio Ingress).
3. → consent-graph-app (ambient waypoint).
4. If request targets a foreign-grantor agreement, app makes cross-region gRPC call to home region's
   consent-graph-app.

### 5.2 East-west traffic
1. Within region: Cilium L4 + Istio ztunnel mTLS.
2. Cross-region: VPC peering + Istio cross-cluster mTLS via SPIRE federation.

### 5.3 Client-region selection for agreement create
- Grantor's home region is determined by grantor tenant's `default_residency_region` setting (from
  identity µservice tenant record).
- If grantor logged in via different region, the request is proxied to grantor's home region for
  the write.

## 6. Failover

### 6.1 AZ outage within region
- Pulsar BookKeeper preserves data (replication factor 3).
- Postgres synchronous standby in another AZ takes over (<60s RTO).
- App pods auto-reschedule (Kubernetes).

### 6.2 Full region outage
- DR region promoted manually via runbook `consent-graph-restart.md` (PHASE-01 manual; PHASE-02
  automated).
- Async-replicated Postgres → up to 30s RPO data loss possible.
- Pulsar topics in failed region unavailable; cross-tenant operations involving that region's
  grantors fail closed (deny-by-default).
- Other regions continue serving their local grantors.

### 6.3 Network partition between regions
- Each region serves its local agreements unimpeded.
- Cross-tenant operations spanning the partition fail closed.
- Revocation topic georeplication paused for partition duration; on heal, replays.

## 7. Latency budgets (per region pair)

| Pair | Postgres write | Pulsar geo lag | Cedar eval | E2E grant→active |
|------|----------------|----------------|------------|-------------------|
| Same region | ≤50ms | n/a | ≤2ms | ≤1s |
| us-east↔us-west | ≤100ms | ≤300ms | ≤2ms | ≤1.5s |
| us↔eu | ≤150ms | ≤500ms | ≤2ms | ≤2s |
| us↔ap | ≤250ms | ≤800ms | ≤2ms | ≤3s |
| eu↔ap | ≤300ms | ≤800ms | ≤2ms | ≤3s |

Worst case (us↔ap grant): well within the 2s p95 SLO for `consent-grant-latency`.

## 8. Sovereignty matrix

| Grantor pack | Grantee region eligibility |
|--------------|----------------------------|
| KR (PIPA strict) | KR only (no cross-border) unless explicit consent + adequacy decision |
| EU (GDPR + Schrems II) | EU + adequacy-decision countries (UK, CH, JP, ...) |
| US-Healthcare (HIPAA) | US only; international transfer requires HIPAA-compliant BAA |
| US (general) | US + GDPR-adequate (with SCC) |
| JP (APPI) | JP + adequacy regions |
| SG / AU / IN / BR | per-pack list |
| AE / KSA | within-Gulf preferred; cross-border requires written agreement |

Encoded in `iac/kustomize/overlays/<pack>/sovereignty-rules.yaml`.

## 9. Capacity

See `capacity-model.md`.

## 10. Pulsar topic counts

For 10M active agreements with mean 2 per (grantor, grantee) pair:
- ~5M unique (grantor, grantee) pairs.
- ~10M projection topics across all regions (each topic in grantor's region).
- Pulsar best practice: ≤100K topics per broker → 100 brokers needed for projection topics alone
  globally. Year-1 capacity planning sizes for 200K topics per region per broker via topic-namespace
  partition.

## 11. DNS + TLS

- consent-graph public REST: `consent-graph.<region>.oya.dev`.
- consent-graph gRPC internal: `consent-graph.<region>.svc.cluster.local`.
- Pulsar admin: `pulsar-admin.<region>.oya.internal` (private subnet only).

## 12. Verification

- Active-active load test: 10K RPS in each region, 10 regions, sustained 1h, p99 ≤2s.
- Failover drill: simulate region outage; failover RTO + RPO measured against targets.
- Partition test: tc-netem on cross-region links; verify fail-closed semantics.

## 13. Risks

- **R**: Citus rebalancing during traffic spike causes write tails.
  **M**: Schedule rebalance during off-hours; emergency rebalance gated behind on-call approval.
- **R**: Geographic-DNS misrouting (e.g., user routed to wrong region).
  **M**: Application-layer region check on JWT tenant binding rejects mismatched routing.
- **R**: Pulsar georeplication backlog during transient WAN issue.
  **M**: Alert on lag >2s; runbook `revocation-incident.md` includes manual catch-up procedure.

## 14. PHASE-02 follow-ups

- Automated regional failover (current PHASE-01 is manual via runbook).
- Adaptive geo-DNS with active health probes (year-2 GA).
- Multi-region Citus federation experiment (research item; not committed).
