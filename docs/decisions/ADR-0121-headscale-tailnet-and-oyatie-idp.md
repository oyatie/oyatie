---
adr: ADR-0121
title: Self-hosted tailnet + Oyatie IdP — public surface contraction and Cloudflare Access retirement
status: Accepted
date: 2026-05-16
authority_window: M3-P02 .. M3-P04
supersedes: []
superseded_by: []
related:
  - ADR-0043 (OpenBao + per-cell HSM)
  - ADR-0044 (service mesh)
  - ADR-0117 (OCI A1 → OKE)
  - ADR-0119 (on-prem k8s stack — kubeadm + containerd + Istio + Envoy)
  - ADR-0120 (Rust-first on-prem tooling)
tags: [identity, network, vpn, headscale, oidc, openbao, mesh, free-tier]
---

# ADR-0121 — Self-hosted tailnet + Oyatie IdP

## Context

The 2026-05-16 deployment session established the on-prem KR cell + the OCI
ap-chuncheon-1 cell. Management surfaces (kms / foundry / ops) were initially
gated by Cloudflare Access (email allowlist over the cloudflared tunnel).
This bakes in three vendor dependencies for the operator path: Cloudflare DNS,
Cloudflare Tunnel, **and** Cloudflare Access. The user directive 2026-05-16
("we are going to use our auth for headscale access — not cloudflare access,
i meant our own auth.oyatie.com") asked for a self-hosted alternative.

In parallel, the user requested a Headscale tailnet control plane on OCI
free tier ("lets do a headscale on one of the micro vm and vpn.oyatie.com")
and explicitly redirected away from running heavy observability on the 1 GB
free micros ("e2micro is not just headscale. should run the observability
there or something like that as well?" → followed by acceptance of the
"observability on on-prem k8s, micros stay minimal" pivot proposed in the
conversation).

This ADR captures the architectural shape that emerges from those three
directives, and the migration sequence that retires Cloudflare Access.

## Decision

### 1. OCI free-tier micros host **identity + watchdog only**, not observability

The 2× E2.1.Micro Always Free quota is utilized as:

| VM | Role | Workloads | RAM committed |
|----|------|-----------|---------------|
| `oyatie-vpn-kr-01` | Public identity / VPN ingress | Headscale 0.27 + embedded DERP relay + node_exporter | ~100 MB |
| `oyatie-watchdog-kr-01` | Tailnet-only outage detector | tailscale client + blackbox_exporter + oya-cloud-watchdog (Rust) + node_exporter | ~80 MB |

Headroom on each VM (~600 MB) is reserved for SPIFFE issuer / step-ca when
M3-P04 lands cross-cell SVID issuance. **No heavy workloads** (Prometheus,
Loki, Grafana) land on these micros — they run on the on-prem k8s cluster
where there is real CPU + RAM + ZFS bulk storage.

#### Why not run observability on the free micros

E2.1.Micro is 1/8 OCPU at AMD EPYC base clock (~0.5–1.0 GHz effective). A
real Prometheus + Loki + Grafana stack GC-stalls inside 1 GB RAM at any
non-trivial load. The "observability survives on-prem outage" argument is
hollow: when on-prem is down there is no data to observe; the relevant
signal is *"on-prem is down"*, which the watchdog binary handles in ~30 MB.
On-prem observability is the right host for the data plane.

### 2. Tailnet control plane is **Headscale** on `vpn.oyatie.com`

- Public IP: reserved + Cloudflare A record `vpn.oyatie.com` (not proxied —
  Headscale needs raw HTTPS gRPC + DERP UDP/3478).
- Node enrollment: OIDC against `auth.oyatie.com` (decision §3).
  Until the IdP is live, pre-auth keys issued by
  `/usr/local/bin/oyatie-headscale-newkey` are the enrollment stop-gap.
- DERP: embedded server, region 999 ("oyatie-kr"), federates with the
  public DERP map for fallback.
- ACL: groups model — single `admins` group containing
  `webservicepost@gmail.com`. Tightened in a follow-up when SPIFFE lands.

### 3. `auth.oyatie.com` is the canonical Oyatie OIDC IdP — self-hosted, **not** Cloudflare Access

Two-phase rollout:

| Phase | Backend | Status |
|-------|---------|--------|
| **A** | OpenBao OIDC provider behind `oya-auth-proxy` (tiny Rust path-rewrite reverse proxy at on-prem :9200) | Wires into existing on-prem OpenBao (kms.oyatie.com); ~80 MB total RSS |
| **B** | `oya-auth-oidc-provider` Rust crate (canonical workspace JWT issuer) | M3-P04 SaaS-platform-preview deliverable; supersedes Phase A |

DNS: `auth.oyatie.com` is a proxied CNAME to the cloudflared tunnel, ingress
maps to `127.0.0.1:9200`. Cloudflare WAF + Rate Limiting + Bot Management
still apply at the edge.

The issuer URL is `https://auth.oyatie.com`. OpenBao's
`vault_identity_oidc_provider` resource accepts a custom `issuer` attribute,
so the discovery document advertises the vanity URL even though OpenBao
internally serves it at `/v1/identity/oidc/provider/oyatie/*`. The path
rewrite happens in `oya-auth-proxy` (Phase A) and natively in
`oya-auth-oidc-provider` (Phase B).

Single OIDC client provisioned for Headscale; future clients (workspace UI,
operator CLI) added as M3-P04 lights up consumers.

### 4. Management surfaces move to **tailnet-only** access; Cloudflare Access is retired

Final public DNS surface contracts to **three hosts**:

- `vpn.oyatie.com` — Headscale (public)
- `api.oyatie.com` — public REST API (already public; hardened by Istio +
  Envoy per ADR-0119)
- `auth.oyatie.com` — OIDC IdP (public; OAuth callbacks must be reachable)

The following Cloudflare-Access-gated subdomains are **removed**:

- `kms.oyatie.com` → tailnet-only at `kms.oyatie.tailnet` (Headscale magic-DNS)
- `foundry.oyatie.com` → tailnet-only at `foundry.oyatie.tailnet`
- `ops.oyatie.com` → tailnet-only at `ops.oyatie.tailnet`

Services are reconfigured to bind only to the `tailscale0` interface on the
on-prem host. Cloudflared ingress rules for those hosts are deleted, and the
Cloudflare Access organization is reduced to **zero applications** (the org
itself remains as the OIDC IdP for any future federation needs).

### 5. Cross-cell trust uses the tailnet + Istio mTLS

- Every node in both cells (on-prem KR primary + OCI nonprod) joins the
  Headscale tailnet on first boot.
- In-cluster service-to-service: Istio PeerAuthentication STRICT mTLS
  (ADR-0119 + the mesh-hardening manifests in `infra/k8s/mesh-hardening/`).
- Cross-cell (e.g., on-prem k8s ↔ OCI E2 watchdog): plain HTTPS over the
  tailnet (the tailnet is the trust boundary; in-flight encryption is
  WireGuard).
- Service identity for finer-grained authz lands in M3-P04 via SPIFFE
  SVIDs issued from auth.oyatie.com (Phase B).

## Migration sequence

Stages run strictly in order; do not retire Cloudflare Access until
Headscale + auth.oyatie.com are verified working.

1. **Headscale provisioning** — apply the OpenTofu changes in
   `infra/oci/compute-aux.tf` + `infra/oci/headscale.tf`. Verify
   `vpn.oyatie.com` resolves and serves the Headscale TLS cert.
2. **Operator enrollment via pre-auth key** — run
   `oyatie-headscale-newkey` on the vpn VM, enroll the on-prem host and
   the operator laptop. Verify tailnet connectivity.
3. **Watchdog provisioning** — apply the watchdog cloud-init; verify the
   watchdog VM joins the tailnet and posts a startup heartbeat to OCI
   Notifications.
4. **OpenBao OIDC + oya-auth-proxy** — configure OIDC provider + client in
   OpenBao; bring up oya-auth-proxy on :9200; verify
   `https://auth.oyatie.com/.well-known/openid-configuration` returns a
   valid discovery document.
5. **Headscale OIDC cutover** — drop the OIDC client_id/secret into
   `/etc/oyatie/oidc-client.env` on the vpn VM; restart headscale; verify
   a fresh enrollment via OIDC succeeds.
6. **Management-surface migration** — rebind on-prem kms / foundry / ops to
   the `tailscale0` interface; remove the Cloudflare Access apps for those
   hosts; remove the cloudflared ingress rules + public DNS records.
   Verify the surfaces are reachable from inside the tailnet and **not**
   reachable from the public internet.
7. **Cloudflare Access org reduction** — once zero applications remain,
   the org can stay (free tier, no cost) for future federation, or be
   deleted in a final cleanup PR.

## Consequences

### Positive

- One auth surface for operators (auth.oyatie.com) — same IdP for tailnet
  enrollment, workspace UI, API access.
- Public attack surface contracts from 6 hosts to 3.
- Vendor surface contracts: Cloudflare Access drops out of the trust path
  (still used for DNS + Tunnel + edge protection).
- Free-tier budget is right-sized: identity + watchdog on micros (~80–100 MB
  each, ~600 MB headroom for SPIFFE); observability on real on-prem hardware.
- Watchdog binary publishes "is on-prem alive" to OCI Notifications — a
  survivable outage signal that doesn't depend on the on-prem cluster.

### Negative

- Operator workflow requires tailnet enrollment for management surfaces.
  A fresh laptop can no longer hit foundry.oyatie.com directly; it must
  first join the tailnet (one-time setup, then ambient).
- Single auth provider = single point of failure. If auth.oyatie.com is
  down, no new node enrollments and no new operator sessions. Mitigation:
  Headscale issued OIDC tokens have configurable lifetime; existing
  enrolled nodes keep working until the next renewal.
- OpenBao OIDC Phase A is a stop-gap; Phase B (`oya-auth-oidc-provider`)
  is the canonical implementation. Carrying both paths in code until
  M3-P04 promotes Phase B is mild technical debt.

### Reversibility

- Re-introducing Cloudflare Access for any surface is a single Cloudflare
  Access app + ingress rule re-add (the tofu state still tracks the org).
- Swapping the IdP backend (Phase A → Phase B) is an issuer-URL constant
  + client re-provisioning, no client (Headscale) reconfiguration needed.
- Tailnet retirement is undoable by re-adding the public CNAMEs and
  cloudflared ingress rules.

## Open questions

- **Watchdog Rust crate publish path**: shipped as a standalone binary via
  GitHub Releases, or pushed to OCIR + pulled by cloud-init? Deferred to
  the M3-P03 OCIR push pipeline ADR.
- **SPIFFE / SVID issuance**: lives on the vpn micro or on a separate
  surface? Resolved in M3-P04 ADR.
- **Cloudflare Access org final disposition**: keep for future federation
  (free) or delete? Resolved after migration step 7 lands.

## Implementation pointers

| Component | Path |
|-----------|------|
| OpenTofu micros + roles | `infra/oci/compute-aux.tf` |
| Headscale NSG + DNS glue | `infra/oci/headscale.tf` |
| Cloud-init: vpn role | `infra/oci/cloud-init/role-vpn.yaml.tftpl` |
| Cloud-init: watchdog role | `infra/oci/cloud-init/role-watchdog.yaml.tftpl` |
| Cloudflare DNS + tunnel | `infra/cloudflare/main.tf` (`auth`, `vpn` records) |
| Watchdog binary (Rust) | `crates/oya-cloud-watchdog/` *(scaffold pending; see ADR-0120 Phase B)* |
| Auth proxy (Rust) | `crates/oya-auth-proxy/` *(scaffold pending; Phase A only)* |
| Canonical IdP (Rust) | `crates/oya-auth-oidc-provider/` *(M3-P04 deliverable)* |
| Mesh-wide hardening | `infra/k8s/mesh-hardening/` |
