# `cloud-network` µservice — Network Engineer FAQ

20 real questions raised against `cloud-network` (the µservice that owns Oyatie's network substrate).

---

**Q1. Does `cloud-network` replace AWS VPC / GCP VPC / Azure VNet?**

It **wraps and unifies**. For tenants on AWS/GCP/Azure, `cloud-network` programs the underlying VPC/VNet via Crossplane via
`cloud-iac`, but exposes a single `cloud-network` API to the tenant. The tenant never directly calls AWS EC2 API or GCP Compute
Engine API for networking — they call `cloud-network`, which translates to provider primitives.

---

**Q2. Why Cilium specifically?**

eBPF lets us implement L3-L7 policy at packet rate (≥ 25 Gbps per node) without sidecar overhead. Cilium clustermesh handles
multi-cluster service discovery natively. Cilium's L7 visibility is critical for the audit-chain anchor — every L7 decision can
be logged with line-rate. Comparable alternatives (Calico-eBPF, Antrea) lack the clustermesh maturity. ADR-0254 binds Cilium 1.18+.

---

**Q3. How is HTTP/3 enforced as default?**

Three places:
1. **Envoy ingress** advertises ALPN `h3, h2, http/1.1` (in that order).
2. **TLS handshake** lists QUIC ALPN before h2; clients that support QUIC pick h3.
3. **The CI lane `lean-a-http3-default`** parses Envoy configs at PR time and refuses ALPN orderings that demote h3.

Tenants can opt out per-route (e.g. for legacy clients) via Cedar permit `cloud_network::Action::AllowHttp2OnlyRoute`.

---

**Q4. What's the tenant tagging mechanism?**

When a pod starts, the Cilium CNI hook injects an eBPF map entry that maps `(pod_ip, pod_port) → (tenant_id, cell_id, pack_id)`.
Every egress packet from the pod is tagged in its socket cookie. The cluster's Cilium policy enforcer reads the tag on packet
ingress and decides. Untagged packets are dropped (demo_trial) or routed to the quarantine network (paid tenant_class) for investigation.

---

**Q5. Can a tenant request a specific CIDR?**

paid tenant_class — yes, via `oya network vpc create --cidr 10.42.0.0/22`. The migration tool can preserve a tenant's existing on-prem CIDR
to ease hybrid connectivity. demo_trial: no (shared VPC).

---

**Q6. How does multi-region work?**

A tenant declares regions; `cloud-network` provisions a VPC per region per cell + cross-region peering (VPC Peering on AWS,
Network Connectivity Center on GCP, Global VNet Peering on Azure). The Cilium clustermesh links the K8s clusters across regions
so service-name resolution works. Cross-region encryption is wireguard + IPsec; latency is provider-dependent (AWS us-east-1 ↔
us-west-2: ~70 ms; intra-region ≤ 4 ms).

---

**Q7. What's the throughput per pod?**

Cilium eBPF: ≥ 25 Gbps per pod on a typical 100 Gbps NIC node (with XDP). With wireguard encryption on, ~12 Gbps per pod
(BoringSSL FIPS shaves another 10 %). DPDK + SR-IOV (paid) — ~70 Gbps per pod.

---

**Q8. How are public IPs managed?**

demo_trial: shared egress IP pool per region. paid tenant_class: per-tenant egress IPs (≤ 4 at paid, ≤ 32 at paid, unlimited at paid).
paid: dedicated /29 or larger per regulatory jurisdiction (e.g. Korean financial regulators require fixed KR IPs).
IP allowlisting for upstream services is a documented config in `cloud_network::Action::ManageEgressIpAllowlist`.

---

**Q9. How does east-west TLS work?**

SPIFFE SVID per pod (X.509 cert chained to the tenant's SPIRE root). Cilium mTLS feature handles handshake termination in the
data path; the application sees plain HTTP. Rotation: 30 m paid, 15 m paid, 5 m paid. SVIDs are short-lived JWTs at demo_trial
(less robust to compromise; reflects lower tenant_class price).

---

**Q10. What's the difference between Cilium policy and Cedar policy?**

Cilium policy (L3-L4-L7 ACLs) is the **enforcement** mechanism in the data path. Cedar policy is the **authority** that the
control plane compiles down to Cilium policy. Tenants author Cedar; `cloud-network` translates to CiliumNetworkPolicy YAML and
applies. Cedar is portable across providers; Cilium policy is Cilium-specific.

---

**Q11. How does this work in air-gapped paid cells?**

Air-gapped cells have no internet egress by default. Specific outbound flows go through a regulator-approved egress proxy (e.g.
KR K-FSI requires KFA-cleared proxies). All upstream services must be in a tenant allowlist; tenant-allowlist is itself Cedar-gated.

---

**Q12. How does flow-log sampling work?**

The Cilium agent emits flow events to a per-node Kafka producer; `cloud-network` consumes and samples. Sampling rate by tier:
- demo_trial: 1 in 1,000 flows (statistical visibility).
- paid: 1 in 100.
- paid: 1 in 10.
- paid: 1 in 1 (every flow).

The full unsampled stream is always available within the cell for `cloud_network::Action::QueryFullFlowStream` (Cedar-gated)
for the past 4 h (paid), 24 h (paid), 7 d (paid).

---

**Q13. How does ECH (Encrypted Client Hello) help?**

ECH (RFC 9460 + draft-ietf-tls-esni-17) prevents on-path observers from seeing the SNI in the TLS handshake. With ECH, a
passive observer sees `cloud-network.oyatie.app` (the public CDN-like front) but not `acme-webapp.acme-software.com` (the
inner tenant hostname). Critical for tenants in jurisdictions with active TLS interception (e.g. China). ECH enabled at paid tenant_class.

---

**Q14. What's the SLO on ingress p99 latency?**

demo_trial: ≤ 350 ms p99 (shared infra). paid: ≤ 80 ms. paid: ≤ 30 ms. paid: ≤ 15 ms. SLOs measured at the public edge to
the first byte from the upstream pod; excludes upstream-side latency.

---

**Q15. How do we handle DDoS?**

Three layers:
1. **L3/L4 volumetric**: provider-side (AWS Shield Standard included; Shield Advanced at paid tenant_class; GCP Cloud Armor at paid tenant_class).
2. **L7 application**: Cilium L7 policy + Envoy rate-limiting; Cedar permit `cloud_network::Action::ActivateL7RateLimit`.
3. **Anomaly response**: detection → `comms-email` + reviewer-agent thread; emergency-blackhole at paid via
   `cloud_network::Action::EmergencyBlackhole`.

---

**Q16. How are private endpoints (e.g. AWS VPC endpoints to S3) handled?**

`cloud_network::Action::EnablePrivateServiceEndpoint` creates the provider-native primitive (AWS Interface Endpoint, GCP PSC,
Azure Private Endpoint) and registers a Cedar resource so policy can reference it. Tenants typically use this for cloud-data
warehouse access without traversing public internet.

---

**Q17. How is IPv6 supported?**

Dual-stack (IPv4 + IPv6) is default at paid tenant_class; IPv6-only is opt-in. Cilium handles dual-stack natively; Envoy listens on both
families. SLAAC + DHCPv6-PD for pod IPv6 allocation. BGP peers can advertise IPv6 prefixes if both sides support it.

---

**Q18. What's the FIPS 140-3 story?**

demo_trial: FIPS off. paid: FIPS optional pack — when enabled, BoringSSL FIPS module 140-3 L1 is used everywhere. paid: FIPS 140-3
L2 mandatory. paid: FIPS 140-3 L3 via Thales HSM-backed BoringSSL. MACsec (L2 encryption) at paid uses HSM-backed keys.

---

**Q19. Where does Foundry hook in?**

Foundry pipelines themselves run in a dedicated Foundry cell with its own VPC; cross-cell traffic from Foundry to tenant cells
is governed by `oyatie.foundry.<pipeline-id>` Cedar principals. Foundry never has direct network access to tenant pods — all
work flows through declared service boundaries on the merge queue admission gate.

---

**Q20. How do I roll back a bad network policy push?**

Cilium policies are versioned in `cloud-network`'s state store. Rollback:
```bash
./bin/oya network policy rollback \
  --tenant <t> \
  --policy <name> \
  --to-version <n>
```

Rollback propagates to all Cilium agents within ≤ 30 s p95. The flow-log emits `network_policy.rollback.applied` events.
Cedar-gated (`cloud_network::Action::RollbackPolicy`).
