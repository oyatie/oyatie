# `cloud-network-dns` µservice — DNS Engineer FAQ

20 real questions raised against `cloud-network-dns` (the µservice that owns Oyatie's DNS substrate).

---

**Q1. Does `cloud-network-dns` replace Route 53 / Cloud DNS / Azure DNS / Cloudflare DNS / NS1 / Akamai DNSi?**

Yes for tenants of Oyatie. The resolver fleet, authoritative name servers, DNSSEC signing, geo-routing, and health-check engine
are all native to `cloud-network-dns`. Tenants can keep their domain registrar (we don't sell domains) but the NS records point
at Oyatie's name servers.

---

**Q2. Why custom rather than vendor DNS?**

Three reasons:
1. **Cedar policy authority** — DNS RR changes are Cedar-gated, allowing per-tenant compliance pack overlays (e.g. KR K-FSI
   requires all DNS records to be reviewer-agent approved before publish).
2. **Co-location with cells** — DNS resolution latency benefits from cell-affinity; vendors are edge-PoP-based and add hop latency.
3. **Sovereign requirements** — paid customers (sovereign / regulated) need air-gapped DNS that doesn't traverse public
   internet; vendors require their cloud accounts.

---

**Q3. What's the anycast architecture?**

Each Oyatie cell advertises a regional anycast range via BGP from its provider's AS. The global anycast IPs (`1.1.42.42`-class)
are advertised from ≥ 16 regions; queries route to the nearest region by BGP. Per-tenant private resolvers use cell-local IPs.

---

**Q4. How is DNSSEC signing performed at paid?**

ZSK + KSK live in HSM partitions (Thales Luna 7 + Utimaco Se Gen2 mix). The signing engine asks the HSM for an ECDSA-P256
signature per RRset; the HSM emits the signature + an attestation receipt. KSK rollover follows RFC 6781 double-signature
method; ZSK rollover is pre-publish with 24-h pre-publish window.

---

**Q5. How does ODoH work?**

ODoH (RFC 9230) interposes a proxy between the client and resolver. The client encrypts the query under the resolver's public key
+ a proxy that strips the client IP. The resolver sees the query but not the IP; the proxy sees the IP but not the query.
`cloud-network-dns` runs both a proxy fleet and a target resolver fleet. Tenants can use the Oyatie proxy or any RFC-9230-compatible
proxy (Apple Private Relay's proxy, Cloudflare's odoh-proxy.cloudflare-dns.com).

---

**Q6. What's the geo-routing source of truth?**

A maintained IP-to-region database compiled weekly from MaxMind GeoIP2 + RIPE NCC routing data + RFC 6724 source-address-selection
hints. Tenants can override per-prefix (`oya dns geo-override set --tenant ... --prefix 203.0.113.0/24 --region NA`).

---

**Q7. How fast is health-check failover?**

demo_trial: not supported. paid: 30 s interval, 3 failures = 90 s. paid: 10 s interval, 3 failures = 30 s. paid: 1 s interval,
multi-region quorum, ≤ 8 s (BGP withdraw timing).

---

**Q8. Can I write a custom record type?**

At paid: register the RR-Type via `oya dns rrtype register`, providing the wire-format codec. The resolver fleet picks up the
codec via plugin. At paid: use RR-Type 65000+ (private use); the codec lives in `crates/cloud-network-dns-rrtype-tenant-<tenant>/`.

---

**Q9. How is DoQ different from DoT?**

DoQ (RFC 9250) is DNS over QUIC instead of TLS-over-TCP. Advantages:
- Connection migration (a phone changing networks doesn't break the session).
- 0-RTT resumption (faster).
- Stream multiplexing (concurrent queries in parallel without head-of-line blocking).

DoQ uses UDP/443 (or 853); DoT uses TCP/853. DoQ is the future per ADR-0253.

---

**Q10. Can I get realtime DNS query telemetry?**

Yes. `cloud_network_dns::Action::SubscribeQueryStream` (Cedar-gated) provides a Kafka stream of per-query events with
`(timestamp, tenant_id, query_name, query_type, response_rcode, latency_micros, source_pop)`. Aggregated dashboards live in
`observability`.

---

**Q11. How does emergency failover work?**

`cloud_network_dns::Action::EmergencyFailover` allows a tenant admin or `oyatie.governance.*` to immediately swap a record set
without health-check confirmation. Used for incident response. Anchored to `audit-chain` with severity `Emergency`.

---

**Q12. How does cell affinity work?**

Each tenant has a "home cell" pinned by `cell.routing.affinity`. DNS queries for that tenant from any anycast IP route to the
home cell's resolvers (via cluster-side routing). On home-cell outage, the BGP-anycast withdraw triggers traffic to the next-closest
cell which has a replicated read-only copy of the tenant zone (replicated within 8 s).

---

**Q13. What's the SLO on authoritative response time?**

demo_trial: ≤ 18 ms p95 global. paid: ≤ 12 ms. paid: ≤ 6 ms. paid: ≤ 3 ms. These are warm-cache; cold cache adds ≤ 8 ms.

---

**Q14. Can I use my own DNSSEC keys (BYO-KSK)?**

paid tenant_class supports BYO-KSK via PKCS#11 import. The KSK material is wrapped + imported into the HSM partition. ZSK remains
auto-managed (with the algorithm matched to the KSK's). BYO-KSK pinning means you control the chain-of-trust; useful for
audit-grade requirements.

---

**Q15. How is reverse DNS (PTR) handled?**

For tenants with dedicated egress IPs (paid tenant_class), the tenant's IP allocation comes with a delegated PTR zone. The tenant manages
PTR records like any other zone. For shared egress IPs (demo_trial), reverse DNS is generic (`unassigned-203-0-113-42.demo_trial.oyatie.net`).

---

**Q16. Can I run a private/internal DNS only?**

Yes — paid tenant_class supports private zones. A private zone is only resolvable from within the tenant's VPC (`cloud-network` integration);
external queries return REFUSED. Useful for internal service discovery.

---

**Q17. What's the DNSSEC algorithm migration story?**

paid supports algorithm 13 (ECDSAP256SHA256). paid adds 14 (ECDSAP384SHA384) and 15 (Ed25519). paid adds experimental
PQC algorithms (Falcon-512, ML-DSA-44) per draft-ietf-dnsop-pqc-dnssec. Algorithm rollover follows RFC 6781 §4.1.4 dual-signature
process: pre-publish new algo for 24 h, swap, post-remove.

---

**Q18. How does this interact with `cloud-iac`?**

When `cloud-iac` provisions a load balancer or CDN endpoint, it can declaratively create matching DNS records via cross-µservice
calls. Records created by `cloud-iac` carry a tag `created_by: cloud-iac`; manual creation is allowed but discouraged for
IaC-managed resources (drift alarm fires).

---

**Q19. Where does Foundry hook in?**

Foundry's CI cells run zone updates as `oyatie.foundry.<pipeline-id>` principals. Cedar permits for Foundry are narrow:
`CreateRecord`, `UpdateRecord` (in dev/staging zones only), `ReadRecord`. Foundry never has production-zone write permits.

---

**Q20. How do I roll back a bad zone change?**

Zones are versioned (every record CRUD increments the SOA serial + the zone version). Rollback:
```bash
./bin/oya dns zone rollback \
  --tenant <t> \
  --zone <zone> \
  --to-version <n>
```

The rollback rewrites the zone to version N, re-signs (DNSSEC), and bumps the SOA serial. Cedar-gated (`cloud_network_dns::Action::RollbackZone`).
Propagation across the resolver fleet ≤ 30 s p95.
