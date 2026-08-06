---
id: ADR-0371
status: Superseded
deciders: founder, council-architecture
date: 2026-05-26
owner: council-architecture
supersedes: []
superseded_by: [ADR-0709]
related: [ADR-0370, ADR-0363, ADR-0043, ADR-0147]
planning_impact: true
milestone: M-PRODUCTION-FIDELITY-SUBSTRATE
depends_on: [ADR-0370]
door: two-way
affected_surfaces:
  crates: []
  microservices: []
  specs: [/infra/cloudflare/, /infra/talos/]
deliverables:
  - id: ADR-0371-D1
    description: "Tunnel + DNS + TCP ingress: a remotely-managed Cloudflare Tunnel routes k8s.oyatie.dev as an L4 TCP stream to the apiserver VIP (tcp://10.211.55.240:6443); DNS CNAME proxied through Cloudflare."
    exit_criteria: "tunnel exists, DNS resolves to the tunnel, ingress maps k8s.oyatie.dev -> tcp://VIP:6443."
    verified_by: "cloudflare API: tunnel + dns_record + configuration created (done 2026-05-26)"
  - id: ADR-0371-D2
    description: "mTLS preserved end-to-end: the L4 TCP route passes the apiserver's client-cert TLS through untouched (no edge TLS termination). An L7/HTTPS ingress is explicitly rejected."
    exit_criteria: "kubectl with the existing client cert authenticates through the tunnel; no cert is terminated at the edge."
    verified_by: "kubectl get nodes succeeds via the tunnel with the unchanged kubeconfig client cert"
  - id: ADR-0371-D3
    description: "Zero-Trust gate: a Cloudflare Access self-hosted app on k8s.oyatie.dev with a Service-Auth policy (service token for CI/agents) fronts the tunnel; optional human SSO/WARP policy."
    exit_criteria: "access without a valid service token (or SSO) is denied at the edge; the service token is least-privilege."
    verified_by: "oya gate validate controlplane-access (to author) + a tokenless connection is rejected"
  - id: ADR-0371-D4
    description: "In-cluster connector as IaC: cloudflared runs as a 2-replica Deployment (infra/cloudflare/cloudflared.yaml), TUNNEL_TOKEN from a Secret sourced from OpenBao/Keychain, dialing outbound only (no inbound ports, no public IP)."
    exit_criteria: "cloudflared Deployment is Healthy and the tunnel shows connected; no inbound port is opened on the host."
    verified_by: "cloudflared /ready healthy + tunnel connections > 0"
  - id: ADR-0371-D5
    description: "Secrets discipline: the scoped Cloudflare API token, connector token, and Access service-token credentials live in Keychain/OpenBao, never in git; the apiserver cert carries k8s.oyatie.dev as a SAN (Talos certSANs)."
    exit_criteria: "no Cloudflare secret is committed; the apiserver cert validates for k8s.oyatie.dev."
    verified_by: "secret-scan clean; openssl s_client shows k8s.oyatie.dev in the apiserver cert SANs"
purpose: Securely serve the Talos Kubernetes control plane at k8s.oyatie.dev with no public IP and no inbound ports, while PRESERVING the apiserver's client-cert mTLS. Decision — Cloudflare Tunnel as an L4 TCP route fronted by Cloudflare Access (Zero Trust, Service Auth) — chosen because it is the only hyperscaler-grade pattern that adds an edge identity factor without terminating the apiserver TLS (which an L7 proxy would, breaking client-cert auth).
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0371: Secure control-plane access via Cloudflare Tunnel (L4) + Access

## Status
Accepted — 2026-05-26.

## Context
The Talos control plane (ADR-0370) runs on a local Parallels network (apiserver VIP
`10.211.55.240:6443`) on a Mac behind NAT with **no public IP**. We need kubectl + automation to
reach it securely from anywhere, gated by strong identity, without port-forwarding or exposing the
host. The apiserver authenticates clients by **mutual TLS (client certificate)** carried in the
kubeconfig. The domain `oyatie.dev` is managed in Cloudflare.

Research (`.omx/plans/cloudflare-controlplane-access.md`) evaluated the options. The load-bearing
constraint: any proxy that **terminates TLS at the edge** (a normal Cloudflare L7/HTTPS ingress)
becomes the TLS client and the kubeconfig client cert never reaches the apiserver — client-cert auth
structurally breaks. A **TCP (L4) route** streams the connection opaquely, so the mTLS handshake
passes through untouched.

## Decision
**Cloudflare Tunnel as an L4 TCP route, fronted by Cloudflare Access (Zero Trust):**

1. **Tunnel (L4 TCP).** A remotely-managed named tunnel (`oyatie-k8s`) with ingress
   `k8s.oyatie.dev -> tcp://10.211.55.240:6443`; a proxied DNS CNAME points the hostname at the
   tunnel. The in-cluster `cloudflared` connector dials **outbound only** — no inbound ports, no
   public IP needed (NAT is a non-issue).
2. **mTLS preserved.** The TCP route does not terminate TLS; the apiserver still sees and validates
   the client cert. The kubeconfig `server:` is `https://127.0.0.1:6443`, reached through a local
   `cloudflared access tcp` listener; the existing client cert/key are unchanged. The apiserver cert
   carries `k8s.oyatie.dev` as a SAN (Talos `cluster.apiServer.certSANs`).
3. **Access (Zero Trust) gate.** A self-hosted Access app on `k8s.oyatie.dev` with a **Service-Auth**
   policy (least-privilege service token for CI/agents) adds an independent edge identity factor;
   optional SSO/WARP policy for humans. Net layering: Access (edge identity) → Tunnel TCP (L4
   passthrough) → apiserver mTLS (workload auth) = two independent factors, zero inbound ports.
4. **IaC + secrets.** `cloudflared` is a 2-replica Deployment (`infra/cloudflare/cloudflared.yaml`)
   with `TUNNEL_TOKEN` from a Secret sourced from OpenBao/Keychain. The scoped Cloudflare API token,
   connector token, and Access service-token credentials live in Keychain/OpenBao — never git.

## Rejected alternatives
- **Cloudflare L7 / HTTPS ingress** — rejected: terminates TLS at the edge → the apiserver client
  cert never arrives → client-cert auth breaks.
- **Cloudflare Spectrum (raw TCP)** — rejected: paid add-on, and unnecessary because the tunnel
  connector dials outbound (no public-IP/NAT problem to solve).
- **Public LoadBalancer / port-forward** — rejected: no public IP, and it exposes the apiserver
  directly without an edge identity factor.
- **VPN-only (e.g., WireGuard) access** — viable but less integrated with the existing Cloudflare
  identity plane; Access service tokens give per-client least-privilege without managing a VPN.

## Consequences
- Positive: the control plane is reachable anywhere with no public IP, no inbound ports, an edge
  Zero-Trust gate, and the apiserver's own mTLS intact — defense in depth. Fully IaC + secrets in KMS.
- Negative/cost: clients need `cloudflared` installed locally and an `access tcp` listener for the
  session; a Cloudflare dependency on the control-plane access path (mitigated: direct VIP access
  still works on-LAN). The cloudflared image must be digest-pinned per the require-signed-images
  policy before production.
- Neutral: this is the access layer above ADR-0370's substrate; it does not change the cluster itself.

## Verification
Per-deliverable `verified_by`. D1 (tunnel/DNS/ingress) and D5 (certSAN in IaC + creds in Keychain) are
done; D2 (kubectl-through-tunnel with mTLS), D3 (Access service-token gate), and D4 (in-cluster
connector Healthy) complete once the cluster's apiserver VIP is live and the connector is deployed.

## References
ADR-0370 (Talos substrate), ADR-0363 (git+Jenkins+GitHub substrate), ADR-0043 (OpenBao KMS),
ADR-0147 (Kata runtime — the workloads behind this control plane). Research + exact configs:
`.omx/plans/cloudflare-controlplane-access.md`. IaC: `infra/cloudflare/cloudflared.yaml`,
`infra/talos/controlplane.patch.yaml` (certSANs).
