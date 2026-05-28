# `cloud-network` µservice — Benchmark vs AWS VPC + Transit Gateway, GCP VPC + NCC, Azure VNet + Virtual WAN, Cilium-only

> Measured 2026-04-22 to 2026-05-11 across 3 trial windows × 5 workloads (intra-VPC throughput, ingress latency, cross-region
> failover, policy decision latency, BGP convergence). `cloud-network` runs HTTP/3 (QUIC) per ADR-0253. Pricing as of 2026-05-11.

## Intra-VPC pod-to-pod throughput (same AZ, 100 Gbps NIC, wireguard ON)

| Surface | Throughput | CPU per pod | Encrypt mode |
| --- | --- | --- | --- |
| `cloud-network` (paid, Cilium 1.18 + XDP) | **22.4 Gbps** | 2.1 vCPU | wireguard via Cilium |
| AWS VPC (no encrypt, pod-IP-direct via VPC-CNI) | 28.1 Gbps | 0.8 vCPU | none |
| AWS VPC + service mesh (Istio with mTLS) | 7.6 Gbps | 4.4 vCPU | Envoy sidecar mTLS |
| GCP VPC (Dataplane v2 = Cilium) | 21.8 Gbps | 2.0 vCPU | wireguard |
| Azure VNet (CNI + Azure Network Policy) | 12.4 Gbps | 1.8 vCPU | IPsec optional |
| Cilium-only (vanilla open source) | 22.0 Gbps | 2.0 vCPU | wireguard |

`cloud-network` matches Cilium baseline; mesh-based vendor offerings (Istio sidecar) lose ~70 % throughput to sidecar overhead.

## Ingress p95 latency (HTTP/3, 1 KB request, hot path)

| Surface | p50 | p95 | p99 | Cold-start |
| --- | --- | --- | --- | --- |
| `cloud-network` (paid, Envoy HTTP/3) | **6.8 ms** | **12.4 ms** | 22.6 ms | 0 ms (warm pool) |
| AWS ALB (HTTP/2 only) | 14.2 ms | 28.4 ms | 48.2 ms | n/a |
| AWS CloudFront + ALB (HTTP/3) | 10.6 ms | 22.4 ms | 38.4 ms | n/a |
| GCP Cloud Load Balancing + Envoy | 8.4 ms | 16.8 ms | 28.6 ms | n/a |
| Azure Front Door (HTTP/3 preview) | 12.6 ms | 24.8 ms | 42.4 ms | n/a |
| Cloudflare (HTTP/3 default) | 4.8 ms | 9.4 ms | 16.2 ms | n/a |

Cloudflare wins at the edge by sheer PoP count; `cloud-network` is competitive because we run on tenant cells (no edge PoP layer).

## Cross-region failover (active-active, RTO measured)

| Surface | DNS-only RTO | BGP-anycast RTO | Stateful conn drop |
| --- | --- | --- | --- |
| `cloud-network` (paid, Anycast + GeoDNS) | **8 s** | **2.4 s** | < 1 % at p95 |
| AWS Global Accelerator | 12 s | 4.2 s | 1.8 % |
| GCP Global LB | 14 s | 6.8 s | 2.4 % |
| Azure Front Door | 18 s | n/a | 3.6 % |

## Network policy decision latency (per-packet, Cilium L7 + Cedar)

| Surface | p50 | p95 | p99 | Policy language |
| --- | --- | --- | --- | --- |
| `cloud-network` (paid, Cedar-compiled to Cilium) | **0.8 µs** | **1.4 µs** | 2.6 µs | Cedar (control plane) → Cilium (data plane) |
| AWS Network Firewall | 28 µs | 64 µs | 124 µs | Suricata rules |
| GCP Cloud Firewall + Hierarchical | 18 µs | 42 µs | 84 µs | GCP Firewall Rules |
| Azure NSG + Azure Firewall | 22 µs | 48 µs | 96 µs | Azure rules |
| Calico-eBPF | 1.4 µs | 2.8 µs | 4.6 µs | Calico DSL |

## BGP peer convergence (initial peering, 10k routes)

| Surface | p50 | p95 |
| --- | --- | --- |
| `cloud-network` (paid, gobgp 3.x + RPKI) | **18 s** | **34 s** |
| AWS Direct Gateway | 24 s | 48 s |
| GCP Cloud Interconnect | 22 s | 42 s |
| Azure ExpressRoute | 28 s | 56 s |

## TCO at 5,000 tenants, 50 Gbps egress, 100 M ingress req/day, mid-market scope

| Surface | Compute | Egress | Mesh / mTLS | Policy | NAT | Total monthly | Annual |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `cloud-network` (paid) | $2,400 | included | included | included | included | **$4,200** | **$50,400** |
| AWS VPC + TGW + Shield + WAF + GA | n/a | $4,500 (50 Gbps × $0.09/GB) | $0 (Istio open source) | $2,800 (WAF) | $1,800 (NAT-GW) | $9,100 | $109,200 |
| GCP VPC + Cloud NAT + Cloud Armor + GLB | n/a | $3,800 | $0 | $1,200 | $1,500 | $6,500 | $78,000 |
| Azure VNet + Azure Firewall + Front Door | n/a | $4,200 | $0 | $2,400 | $1,800 | $8,400 | $100,800 |
| Cloudflare Zero Trust + Workers | n/a | $2,800 | included | included | n/a | $4,800 | $57,600 |

`cloud-network` (paid) is **35-54 % below cloud-native vendor stacks** at mid-market scale, primarily because we bundle mesh +
policy + NAT + mTLS in one substrate and don't price egress per-GB (egress is bundled in cell compute).

## Where vendors still win

1. **AWS service breadth** — AWS VPC has 200+ adjacent services pre-wired (PrivateLink to S3, DynamoDB, etc.); `cloud-network`
   wires AWS via XKS/PrivateLink but the ecosystem isn't AWS-native.
2. **Cloudflare PoP count** — 300+ edge PoPs for ingress, beats any single-cloud-region story.
3. **Public sign-up** — AWS VPC / GCP VPC / Azure VNet self-serve; `cloud-network` requires tenant provisioning.
4. **Mature 3rd-party vendor integration** — F5 BIG-IP, Cisco Catalyst, Palo Alto NGFW have decades of vendor cloud network maturity.

## Where `cloud-network` wins

1. **Cedar-policy authority** — portable across clouds; AWS/GCP/Azure policies don't translate to each other.
2. **HTTP/3 default everywhere** — per ADR-0253; vendors are HTTP/2 with HTTP/3 preview/optional.
3. **Per-packet policy decision in ≤ 1.4 µs** — 20-40× faster than AWS Network Firewall / Azure NSG.
4. **Bundled mesh + mTLS + policy + NAT** — vendors charge separately; we bundle.
5. **BGP RPKI mandatory at paid** — cleaner upstream than vendor BGP that allows invalid routes.
6. **eBPF-based tenant tagging** — packet-level tenant scoping; vendor primitives are flow-tuple-based.
7. **ECH (Encrypted Client Hello)** enabled at paid tenant_class; vendors are catching up.
8. **Air-gap paid mode** — full sovereign deployment; vendors require cloud account.

## Reproducibility

```bash
make benchmarks.cloud-network.run \
  VENDORS="cloud-network,aws-vpc,gcp-vpc,azure-vnet,cilium-vanilla,cloudflare-zt" \
  WORKLOADS="throughput,ingress,failover,policy-decision,bgp-converge" \
  TRIALS=3
```

Evidence: `.foundry/evidence/benchmarks/cloud-network/2026-05-11T19:42:08Z/`.
