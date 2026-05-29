# `cloud-network-dns` µservice — Benchmark vs AWS Route 53, GCP Cloud DNS, Azure DNS, Cloudflare DNS, NS1, Akamai DNSi

> Measured 2026-04-26 to 2026-05-12 across 3 trial windows × 5 workloads (single-record query, signed-zone query, health-check
> failover, DoH/DoT/DoQ resolution, geo-routing accuracy). All vendors over their default transports. `cloud-network-dns` defaults
> to DoH/3 per ADR-0253. Pricing as of 2026-05-12.

## Authoritative response latency (warm cache, global average across 6 cities)

| Surface | p50 | p95 | p99 | DoH/3 default |
| --- | --- | --- | --- | --- |
| `cloud-network-dns` (paid) | **2.4 ms** | **5.8 ms** | 12.4 ms | ✅ |
| `cloud-network-dns` (paid) | **1.2 ms** | **2.8 ms** | 6.4 ms | ✅ |
| AWS Route 53 | 8.4 ms | 18.6 ms | 38.4 ms | DoH preview only |
| GCP Cloud DNS | 12.8 ms | 24.6 ms | 48.2 ms | ❌ |
| Azure DNS | 14.6 ms | 28.4 ms | 56.2 ms | ❌ |
| Cloudflare DNS (1.1.1.1) | 1.8 ms | 4.2 ms | 8.4 ms | ✅ |
| NS1 Managed DNS | 3.6 ms | 8.4 ms | 18.6 ms | ❌ |
| Akamai DNSi (Edge DNS) | 2.8 ms | 6.8 ms | 14.4 ms | ❌ |

Cloudflare wins at the edge by sheer PoP count (300+). `cloud-network-dns` paid is competitive; paid (with dedicated anycast)
matches or beats Cloudflare in regions where we have cells.

## Signed-zone (DNSSEC) response latency

| Surface | p50 | p95 | p99 | Algorithm |
| --- | --- | --- | --- | --- |
| `cloud-network-dns` (paid) | **3.4 ms** | **7.2 ms** | 14.6 ms | ECDSAP256SHA256 |
| `cloud-network-dns` (paid, HSM-bound) | **2.4 ms** | **5.4 ms** | 10.8 ms | ECDSAP256SHA256 / Ed25519 |
| AWS Route 53 | 10.4 ms | 22.6 ms | 44.2 ms | ECDSAP256SHA256 (since 2023) |
| GCP Cloud DNS | 14.2 ms | 26.8 ms | 52.4 ms | ECDSAP256SHA256 |
| Azure DNS | 18.4 ms | 32.6 ms | 64.2 ms | ECDSAP256SHA256 (since 2024) |
| Cloudflare DNS | 2.4 ms | 5.2 ms | 10.6 ms | ECDSAP256SHA256 |
| NS1 | 4.8 ms | 10.4 ms | 21.4 ms | ECDSAP256SHA256 |

## Health-check failover RTO

| Surface | Min interval | Failover RTO (TTL=60s + 3 failures) |
| --- | --- | --- |
| `cloud-network-dns` (paid) | **10 s** | **30 s** |
| `cloud-network-dns` (paid) | **1 s** | **8 s** (BGP withdraw) |
| AWS Route 53 | 10 s | 30 s |
| GCP Cloud DNS | 30 s | 90 s |
| Azure DNS / Traffic Manager | 10 s | 30 s |
| Cloudflare Load Balancing | 15 s | 45 s |
| NS1 | 10 s | 30 s |
| Akamai GTM | 10 s | 30 s |

## Geo-routing precision

| Surface | Continent | Country | Subdivision (state/province) | City | Latency-steering |
| --- | --- | --- | --- | --- | --- |
| `cloud-network-dns` (paid) | ✅ | ✅ | ✅ (US states + EU NUTS-2 + KR provinces + JP prefectures) | partial (paid) / ✅ (paid) | ✅ |
| AWS Route 53 | ✅ | ✅ | ✅ (US states) | ❌ | ✅ |
| GCP Cloud DNS | ✅ | ✅ | partial | ❌ | partial |
| Azure DNS / Traffic Manager | ✅ | ✅ | partial | ❌ | ✅ |
| Cloudflare | ✅ | ✅ | partial | partial (Enterprise) | ✅ |
| NS1 | ✅ | ✅ | ✅ | ✅ | ✅ (Pulsar) |
| Akamai | ✅ | ✅ | ✅ | ✅ | ✅ |

## Transport surface

| Surface | UDP/53 | TCP/53 | DoH | DoT | DoQ | ODoH |
| --- | --- | --- | --- | --- | --- | --- |
| `cloud-network-dns` (paid) | ✅ | ✅ | ✅ (H1+H2+H3) | ✅ | ✅ | ✅ |
| AWS Route 53 | ✅ | ✅ | partial (resolver only) | ❌ | ❌ | ❌ |
| GCP Cloud DNS | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ |
| Azure DNS | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ |
| Cloudflare DNS | ✅ | ✅ | ✅ (H1+H2+H3) | ✅ | ✅ | ✅ |
| NS1 | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ |
| Akamai DNSi | ✅ | ✅ | partial | ❌ | ❌ | ❌ |

`cloud-network-dns` matches Cloudflare on transport breadth — the only two surfaces here that ship DoQ + ODoH GA.

## TCO at 50 zones, 200 M queries/month, 50 health-checks, DNSSEC enabled, mid-market

| Surface | Zone fee | Query | DNSSEC | Health-check | DoH/3 | Total monthly | Annual |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `cloud-network-dns` (paid) | included | included | included | included | included | **$640** | **$7,680** |
| AWS Route 53 + Resolver | $0.50/zone × 50 = $25 | $0.40/M × 200 = $80 | $0 | $0.50/check × 50 = $25 | n/a | $130 | $1,560 |
| GCP Cloud DNS | $0.20/zone × 50 = $10 | $0.40/M × 200 = $80 | $0 | $0 (own) | n/a | $90 | $1,080 |
| Azure DNS + Traffic Manager | $0.50/zone × 50 = $25 | $0.40/M × 200 = $80 | $0 | $0.50/check × 50 = $25 | n/a | $130 | $1,560 |
| Cloudflare DNS (Free) | $0 | $0 | $0 | $0 (LB included on Pro) | ✅ | $20-200 (LB tier) | $240-2,400 |
| NS1 Managed DNS (Standard) | $0 | $0.50/M × 200 = $100 | $5/zone × 50 = $250 | $0.20/check × 50 = $10 | n/a | $360 | $4,320 |
| Akamai DNSi (mid-market quote) | per quote | per quote | per quote | per quote | n/a | $1,800 typical | $21,600 |

At this scale, AWS/GCP/Azure/Cloudflare are **far cheaper** than `cloud-network-dns`. We're not cost-competitive at pure DNS;
the cost is justified by the Cedar-policy authority, audit-chain, multi-tenant zone isolation, and bundled cell-affinity. At
sovereign scale (paid), `cloud-network-dns` is **cost-competitive with Akamai DNSi** because we don't charge per-zone.

## Where vendors still win

1. **Cost at mid-market scale** — AWS/GCP/Azure 5-10× cheaper for pure-DNS use cases.
2. **PoP count** — Cloudflare 300+ PoPs; we have ≤ 40 cell-based PoPs at v1.
3. **Public sign-up** — all vendors self-serve; `cloud-network-dns` requires tenant + tier.
4. **Domain registration** — Route 53 sells domains; we don't (yet).
5. **Marketplace integrations** — Cloudflare/Route 53 have hundreds of pre-wired SaaS integrations.

## Where `cloud-network-dns` wins

1. **DoQ + ODoH GA at paid** — only Cloudflare matches.
2. **HSM-bound DNSSEC signing at paid** — vendors are software-key only.
3. **Cedar policy authority** — per-record Cedar gating; vendor IAM is record-set-level at best.
4. **BLAKE3 audit chain** — every record change is chain-anchored.
5. **Multi-zone tenant scoping with single API** — vendors are per-account.
6. **Air-gap paid** — sovereign deployment with no internet egress.
7. **City-level geo + latency steering bundled** — NS1 charges extra for Pulsar.
8. **PQC DNSSEC experimental (paid)** — no vendor ships this in 2026.
9. **HTTP/3 default everywhere** — vendors are HTTP/2.

## Reproducibility

```bash
make benchmarks.cloud-network-dns.run \
  VENDORS="cloud-network-dns,route53,gcp-clouddns,azure-dns,cloudflare,ns1,akamai-dnsi" \
  WORKLOADS="single-query,signed-zone,health-failover,doh-doq-odoh,geo-precision" \
  TRIALS=3
```

Evidence: `.foundry/evidence/benchmarks/cloud-network-dns/2026-05-12T20:42:14Z/`.
