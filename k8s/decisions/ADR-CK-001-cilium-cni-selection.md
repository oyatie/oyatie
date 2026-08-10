---
adr_id: ADR-CK-001
scope: k8s
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0121, ADR-0131, ADR-0254]
doc_status: published
---

# ADR-CK-001 — CNI selection: Cilium 1.18 over Calico / Canal / Flannel / Antrea

## Status
Accepted 2026-05-20 by axis-cloud + axis-network. Lasts until at least Kubernetes 1.39 LTS.

## Context

The on-prem Kubernetes substrate ships a CNI per ADR-0121's "vanilla upstream + minimal patch surface" stance. We evaluated five CNIs in early Wave-3-C for the paid tenant_class production envelope: Cilium 1.18, Calico 3.29 (with Felix dataplane), Canal (Flannel + Calico-policy), Flannel 0.26 alone, and Antrea 2.4 (OVS-based).

The evaluation criteria, in priority order:

1. eBPF dataplane availability for sub-1ms in-cluster latency at 250 k pps per node.
2. NetworkPolicy semantics that map cleanly to Cedar fragments (per ADR-0243).
3. IPv6 + IPv4 dual-stack support with no per-cluster surgery.
4. Service-mesh integration shape (Istio + Cilium clusterMesh interoperability).
5. Multi-cluster federation primitives (relevant for paid on-prem-connected cell_topology per `capability-tiers/tier-matrix.md`).
6. Operational maturity (CVE response time, LTS branch lifetime, conformance lab presence).
7. Licence (CNCF / Apache-2.0 hard preference).

## Decision

Use **Cilium 1.18** as the default CNI for all `cloud-k8s` profiles. Specifically:

- demo_trial: Cilium 1.18 in vxlan mode (no eBPF host routing; reduces kernel-version dependency for non-prod).
- paid dedicated-cloud + paid on-prem-connected: Cilium 1.18 with eBPF host routing + kube-proxy replacement (`kubeProxyReplacement=strict`).
- paid compliance_pack: Cilium 1.18 with WireGuard transparent encryption (FIPS 140-3 mode where the pack requires; `encryption.wireguard.enabled=true`).

Reject Calico, Canal, Flannel, Antrea for the reasons in §Consequences.

## Detailed rationale

### Why Cilium over Calico

Calico's Felix dataplane is mature (Project Calico has shipped since 2016) but its eBPF dataplane (`Felix-eBPF`) was promoted to GA late vs Cilium's, and the operational surface is larger — Calico's per-node Typha + Felix split adds two control daemons per node where Cilium ships one (cilium-agent). Calico's NetworkPolicy implementation supports CIDRs, named ports, and namespaceSelectors; so does Cilium's. Cilium adds L7 policy (HTTP path / method / header matching) via Envoy that Calico requires `bgp-route-reflector + envoy-sidecar` extra setup for. We use L7 NetworkPolicy in three places already (`policy/cluster-isolation.md` §"Per-tenant HTTP path quarantine"); Cilium gives us this for free.

Calico's BGP-as-default for L3 is excellent for ToR-routed datacentres but oyatie's `cloud-iac` substrate runs vxlan for cross-rack east-west; we don't get the Calico BGP benefit at our shape.

### Why not Canal

Canal = Flannel for the dataplane + Calico for the policy. Two control surfaces, one CNI. The operational story is "you get the worst of both". In Rancher RKE2 it's the default and works; for oyatie's substrate the additional ops surface is not justifiable.

### Why not Flannel alone

Flannel does not implement NetworkPolicy. ADR-0243's default-deny requires NetworkPolicy enforcement at the CNI; Flannel alone makes oyatie's deny posture impossible without an out-of-band enforcement layer. Reject.

### Why not Antrea

Antrea is OVS-based, which is excellent for OpenStack-shaped fabrics but oyatie's cells are Kubernetes-native, not OpenStack-native. OVS introduces a userspace fastpath (ovs-vswitchd) that is one more daemon to keep healthy at scale. Antrea's NetworkPolicy implementation is mature and on-par with Cilium; the deciding factor is the dataplane shape (eBPF vs OVS).

### Service-mesh integration

Cilium's `clusterMesh` and Istio's multi-cluster federation interoperate cleanly at Cilium 1.18+ via shared service-import semantics. We tested at Wave-3-C with a 3-cluster federation; cross-cluster service-resolution latency p99 was 4.2 ms (vs Istio-only at 6.8 ms because Istio resolved via DNS roundtrips that Cilium clusterMesh short-circuits).

Calico + Istio works but requires a Calico-side install of the `calico-istio-csi` driver that adds another ops surface.

### IPv6 dual-stack

All five candidates support dual-stack at the K8s level. Cilium's dual-stack maturity is the highest by issue-tracker volume (we surveyed 2025-Q4 + 2026-Q1 IPv6-tagged issues; Cilium had 18 closed in that window vs Calico 12, Antrea 4, Canal 0, Flannel 0). The KR-PIPA + IN-DPDPA packs require IPv6 by 2027 per `iac/pack-roadmap.yaml`; this matters.

### Encryption

For paid compliance_pack tier, we need transparent in-cluster encryption (WireGuard or IPsec). Cilium ships WireGuard transparent encryption; toggle via `encryption.wireguard.enabled=true`. Calico ships WireGuard support; equivalent. Antrea ships IPsec; equivalent. The deciding factor is performance — Cilium's WireGuard mode benchmarked at 23.4 Gbps node-to-node (vs Calico's at 19.1 Gbps and Antrea's IPsec at 14.2 Gbps; same hardware, see `benchmarks/cni-encryption-throughput.md` for the harness).

## Consequences

Positive:

- One CNI across all tiers reduces ops surface.
- L7 NetworkPolicy enables per-tenant HTTP path quarantines without sidecar injection (demo_trial tenant_class benefit).
- eBPF dataplane keeps in-cluster latency under our SLO with significant margin (measured 0.4 ms p99 for in-rack pod-to-pod at paid dedicated-cloud).

Negative:

- Cilium 1.18 requires Linux kernel ≥ 5.10 for eBPF mode; we mandate 6.6 LTS per the preflight, so this is moot in practice but constrains the hardware refresh roadmap.
- The cilium-agent DaemonSet runs as privileged (NET_ADMIN, SYS_MODULE). Pod-Security-Standards `restricted` cannot apply to it; we ship a per-namespace exemption documented in `policy/cluster-isolation.md` §"CNI privileged daemonset exemption".
- Cross-vendor migration (Calico → Cilium, see `migration-playbooks/from-rancher-rke2.md`) is non-trivial because Cilium's eBPF connection-tracking state is not import-compatible with Calico's conntrack. Cutover involves transient connection-drop for established TCP.

## Compliance

Cilium 1.18 has an EAL2-equivalent independent security review (Cure53, 2025-Q3). Calico Enterprise has FedRAMP High; Cilium Enterprise (Isovalent) has FedRAMP Moderate-equivalent + ongoing review. For paid compliance_pack tier in FedRAMP-bound packs (US-GOV-default), we use Cilium Enterprise; for other tiers, upstream OSS Cilium suffices.

## Migration triggers

Re-open this ADR if any of:

- Cilium 1.18 has > 3 unpatched CVEs of CVSS ≥ 7.0 unresolved for > 60 days.
- Cilium upstream announces dataplane revamp incompatible with our 1.18 LTS baseline.
- Antrea introduces an eBPF dataplane GA (would re-open the OVS-vs-eBPF question).
- A pack mandates a CNI we don't ship (e.g., a CN-PIPL-bound pack requires Calico-Enterprise per Chinese certification rules; we'd add Calico as a pack-bound exception, not replace Cilium globally).
