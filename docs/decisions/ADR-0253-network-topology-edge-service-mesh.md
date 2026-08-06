---
id: ADR-0253
status: Accepted
planning_impact: true
date: 2026-05-20
owners:
  - council-architecture
  - council-security
  - council-privacy
  - axis-network
  - axis-cloud-k8s
  - axis-edge
  - ops-sre-reliability
  - ops-security
  - ops-compliance
supersedes: []
amends: []
superseded_by: []
amended_by:
  - ADR-0565
related:
  - ADR-0009-cell-architecture-per-tenant-per-region.md
  - ADR-0010-regional-pack-architecture.md
  - ADR-0028-audit-chain-merkle-sealed.md
  - ADR-0043-production-cell-hardening.md
  - ADR-0044-service-mesh-and-mtls.md
  - ADR-0049-cross-region-replication-and-residency.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0117-oke-parity.md
  - ADR-0121-onprem-k8s-stack-kubeadm-containerd-istio-envoy.md
  - ADR-0128-hyperscaler-architecture-invariants.md
  - ADR-0131-per-microservice-flat-layout.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0148-service-mesh-cilium-ambient-layered.md
  - ADR-0149-api-gateway-vs-service-mesh-separation.md
  - ADR-0150-cursor-pagination-canonical.md
  - ADR-0153-observability-backplane-layering.md
  - ADR-0183-policy-engine-separation-cedar-app-authz-kyverno-admission.md
  - ADR-0211-in-house-tech-stack-policy.md
  - ADR-0223-oya-git-drop-in-surface-with-explicit-policy-verbs.md
  - ADR-0240-sovereign-cloud-per-regional-pack.md
  - ADR-0241-dr-business-continuity-portfolio-policy.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0246-policy-engine-substrate-promotion.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0252-time-coordination-distributed-consistency.md
  - ADR-0565-zero-graphql-in-the-owned-api-surface.md
related_specs:
  - /specs/platform-architecture.json
  - /specs/network-topology.json
  - /specs/microservices/edge-gateway.json
  - /specs/microservices/dns.json
  - /specs/microservices/service-mesh.json
related_memory:
  - feedback_quality_performance_scalability_bar
  - feedback_bominal_inheritance_precedence
  - feedback_autonomous_implementation_artifacts
  - feedback_no_silent_regression
  - feedback_oya_git_canonical_2026_05_18
doc_class: Architecture-Decision-Record
keystone_bundle: 2026-05-20-foundational-doctrine
keystone_position: 12-of-14
purpose: >
  Establish the canonical network topology spanning planetary apex DNS,
  edge POPs, per-cell ingress termination, the Cilium-ambient service
  mesh inside cells (per ADR-0148), workload identity via SPIFFE/SPIRE,
  TLS 1.3 with post-quantum hybrid key exchange, HTTP/3 client-side,
  inter-cell + cross-provider mesh, and the migration path from
  Cloudflare-hosted edge to self-hosted Pingora POPs by Year 3+. This
  ADR sets the broader network shape around the in-cell service mesh
  already chosen in ADR-0148.
enforcement_status: advisory-until-edge-and-mesh-substrates-land
enforced_by:
  - oya gate validate network-topology-coherence
  - oya gate validate edge-pop-presence
  - oya gate validate tls13-only
  - oya gate validate spiffe-svid-coverage
  - oya gate validate post-quantum-hybrid-readiness
---

# ADR-0253: Network topology — Anycast apex + edge POPs + Cilium ambient service mesh inside cells

## Status

Proposed — 2026-05-20.

Bundled with the 14-ADR foundational keystone set (ADR-0242 through
ADR-0255 inclusive) landing as a single multispectrum-reviewed PR.
Keystone position 12-of-14. Each keystone references the others;
partial acceptance is rejected because the doctrines are mutually
reinforcing.

Enforcement is `advisory-until-edge-and-mesh-substrates-land`. The
ADR's invariants accept in text immediately, but the CI lanes that
enforce them promote to BLOCKER only after:

1. `microservices/edge-gateway/` admits a Cloudflare-Workers-backed
   edge substrate with per-zone configuration baselined and
   cosign-attested per ADR-0223.
2. `microservices/dns/` admits an Anycast + GeoDNS topology with
   per-cell health checks and a documented failover matrix.
3. `microservices/service-mesh/` admits Cilium 1.16 LTS + Istio
   Ambient 1.24 LTS waypoints per ADR-0148, with SPIRE 1.10+ trust
   federation enabled across cells.
4. `microservices/policy-engine/` admits a Cedar fragment library
   that consumes SPIFFE-ID as the principal claim on every request
   per ADR-0243.

Until those four bootstrap items land, validators emit findings
without failing CI. Post-bootstrap, the lanes promote to BLOCKER.

## Date

2026-05-20.

## Context

### Why a network-topology keystone is required now

The portfolio's prior network artifacts are correct in scope but
fragmented:

- **ADR-0044 (service mesh + mTLS, 2026-05-14)** established mTLS as
  the canonical wire posture and Istio as the initial mesh control
  plane.
- **ADR-0121 (on-prem K8s stack, 2026-05-16)** picked vanilla kubeadm
  + containerd + Istio + Envoy as the on-prem K8s substrate, with the
  understanding that mesh choices would later harmonise across cloud
  + on-prem.
- **ADR-0145 (inter-µservice communication reform, 2026-05-18)**
  replaced the universal-mediator rule with three weaker invariants
  (audit, tracing, ontology projection), permitting direct sibling-
  µservice gRPC under mTLS.
- **ADR-0148 (service mesh canonical: Cilium L3/L4 + Istio Ambient
  L7, 2026-05-18)** chose Cilium ambient + Istio Ambient waypoints as
  the layered in-cell mesh, with zero feature overlap per layer.
- **ADR-0149 (API gateway north-south vs service mesh east-west,
  2026-05-18)** drew the boundary between north-south ingress and
  east-west service mesh, but left open the question of *what the
  north-south path looks like at planetary scale*.
- **ADR-0240 (sovereign cloud per regional pack, 2026-05-18)** set
  per-pack provider matrices but did not specify how clients reach
  the platform from anywhere on the planet.
- **ADR-0241 (DR + business-continuity portfolio policy, 2026-05-18)**
  set per-µservice DR tiers but did not specify GeoDNS failover.
- **ADR-0242 (oyatie-is-a-tenant doctrine, 2026-05-20)** establishes
  uniform tenant treatment for `oyatie` as a first-class tenant.
- **ADR-0243 (Cedar as universal gate, 2026-05-20)** establishes
  Cedar as the universal authorization gate; the gate requires a
  principal claim, which in turn requires a workload identity.
- **ADR-0248 (Amazon-shape cellular architecture, 2026-05-20)**
  establishes cellular architecture; the inter-cell traffic shape
  must be specified.

None of those ADRs defined:

- the **apex DNS** topology (Anycast + GeoDNS at the planet's edge);
- the **edge layer** topology (CDN + DDoS + WAF + bot mitigation +
  TLS-terminated client connections at POPs near the user);
- the **post-quantum** posture (when to add hybrid KEX);
- the **HTTP/3 (QUIC) client-side** decision;
- the **inter-cell trust** mesh (SPIRE federation across cells);
- the **cross-provider** mesh (per ADR-0240, when a tenant pinned to
  EU pack on AWS-Frankfurt talks to a tenant pinned to KR pack on
  NHN-Cloud-Pangyo via an opt-in inter-provider tunnel);
- the **migration path** from Cloudflare-hosted edge to self-hosted
  Pingora POPs that ADR-0211 (in-house tech stack policy) makes
  inevitable at scale;
- the **HTTP API surface canonical** (REST 3.2 / gRPC) at the public-facing edge. GraphQL
  Federation v2 is historical rejected context only; ADR-0565 removed it from the owned surface.

This keystone closes those gaps.

### Why now — the forcing functions

Three forcing functions converge on 2026-05-20:

1. **Tenants reach the platform from anywhere on the planet.** The
   `feedback_quality_performance_scalability_bar` memory establishes
   the bar as Stripe + Palantir + Linear + hyperscaler-grade. A
   tenant in Berlin loading the Workflow Studio (per ADR-0136
   dissolution + ADR-0255 Intelligence rewrite + the Workflow Studio
   scope memory) cannot pay 200ms+ of trans-Atlantic round trip on
   the first byte. The edge must be near the user.
2. **Latency budgets demand edge presence.** P99 first-byte budgets
   for human-facing surfaces (the Consumer Brand Surface layer per
   ADR-0255) are ≤ 100ms in cell, ≤ 200ms cross-cell, ≤ 50ms first
   byte (favouring TCP-cwnd-already-grown sessions on HTTP/2 + HTTP/3
   resumption). Without an edge POP within ~30ms of every population
   centre, those budgets are unmeetable.
3. **DDoS attacks demand absorption at edge.** A volumetric DDoS
   attack (1+ Tbps observed against Cloudflare in 2024-2025; AWS
   Shield team observed 2.3 Tbps attack in 2020; observed 25.3
   billion-requests-per-second HTTP/2 Rapid Reset attack against
   Google/Cloudflare/AWS in October 2023) cannot be absorbed at
   per-cell ingress; it must be absorbed at the planetary edge before
   it hits backbone uplinks to the cell.
4. **Modern protocols (HTTP/3, TLS 1.3, post-quantum) demand
   consolidated mesh.** ML-KEM-768 (NIST FIPS 203, August 2024) is
   shipping at Cloudflare + AWS s2n-tls + OpenSSL 3.x; the
   "harvest-now-decrypt-later" threat means PQ-hybrid KEX has a
   2026-2028 deployment window before standards bodies start
   demanding it for regulated data.
5. **ADR-0148 already chose Cilium ambient** as the in-cell mesh.
   This ADR sets the broader network topology around it without
   contradicting ADR-0148.

### Hyperscaler reference matrix

| Hyperscaler | Apex DNS | Edge layer | TLS termination | Service mesh inside | Workload identity | PQ posture |
|---|---|---|---|---|---|---|
| Cloudflare | Anycast at 300+ POPs | Cloudflare Workers (V8 isolates) + CDN + WAF + Bot | Per-POP TLS 1.3 | Internal mesh (Pingora, Linkerd-like) | Internal Cloudflare identity | ML-KEM-768 + X25519 hybrid since Q3 2024 (per Cloudflare blog) |
| Google | Anycast at 100+ POPs | Google Front End (GFE) | Per-POP TLS 1.3 | Bandwidth Enforcer + ALTS (workload identity) + Istio (GKE) | ALTS / SPIFFE for K8s | NIST ML-KEM in GFE since 2024 |
| AWS | Route 53 (Anycast) | CloudFront (350+ POPs) + AWS Shield + WAF | Per-POP TLS 1.3 | App Mesh (Envoy) on EKS | IAM Roles for Service Accounts (IRSA) | s2n-tls PQ hybrid since 2024 |
| Microsoft | Azure DNS (Anycast) | Azure Front Door + Azure CDN | Per-POP TLS 1.3 | Open Service Mesh (Envoy) on AKS | Workload Identity (federated) | TLS 1.3 PQ in S2C path 2025 |
| Apple | Apple Edge Cache | Apple Edge Cache (internal CDN; ~150 POPs) | Per-POP TLS 1.3 | Internal mesh; not public | Internal Apple workload identity | Stated PQ commitment WWDC 2024 |
| Stripe | Cloudflare for stripe.com; Route 53 for API | Cloudflare in front of api.stripe.com | Per-POP TLS 1.3 | Twirp/gRPC + Envoy + IRSA | IRSA + custom JWT | Adopting PQ via Cloudflare's stack |

The pattern is unambiguous: **mature platform companies operate
Anycast apex + planetary edge POPs + per-POP TLS termination + in-cell
service mesh + workload identity (SPIFFE or equivalent) + post-quantum
hybrid KEX rolling out 2024-2026.**

Oyatie matches this shape, with one engineered difference: at Year 3+
scale, the edge migrates from Cloudflare-hosted to self-hosted
Pingora POPs (Cloudflare's open-sourced Rust proxy, released 2024),
matching ADR-0211 in-house preference. Until Year 3, oyatie rides on
Cloudflare's edge.

### Why edge is NOT in K8s

A common architectural mistake is treating "the edge" as another K8s
cluster. Three reasons it isn't:

1. **POP density.** Cloudflare runs ~300 POPs globally; Akamai runs
   ~4,000. K8s clusters require etcd quorum + kube-apiserver +
   kube-controller-manager + scheduler + cluster autoscaler + Cilium
   agent + Istio control plane; the resource floor is ~2 GB RAM + 4
   vCPU per cluster. 300 POPs × 2 GB = 600 GB RAM just to run empty
   K8s control planes. V8 isolates (Cloudflare Workers) start in
   ~5ms with ~3 MB memory per isolate; ~3000 isolates fit in 9 GB.
2. **Cold-start latency.** K8s Pod cold-start is dominated by image
   pull (seconds) and Pod scheduling (hundreds of milliseconds). V8
   isolate cold-start is 5ms. Pingora (Rust async I/O on epoll) starts
   in single-digit milliseconds for new connections, with zero cold
   start for the long-lived proxy process.
3. **Network stack.** Edge POPs are network appliances first
   (Anycast IP advertisement, TLS termination, HTTP parsing,
   request routing); K8s schedules workloads, not network appliances.
   Cilium + kube-proxy are designed for east-west traffic; Pingora /
   V8-isolates are designed for north-south.

The decision: **edge = V8 isolates or Pingora; NOT K8s. Server-side =
always K8s Pods.** This boundary is the canonical edge-to-cell
demarcation line.

## Decision

### D-1. Apex DNS — Anycast + GeoDNS, externally hosted first, self-hosted Year 3+

The apex DNS for oyatie operates as **Anycast + GeoDNS at the planet's
edge**.

**Year 1-2 (externally hosted):**

- **Primary registrar + DNS provider:** Cloudflare DNS (Anycast across
  300+ POPs) + AWS Route 53 (Anycast across 100+ POPs) as **dual
  authoritative** for redundancy. Per ADR-0240 sovereign-cloud-per-
  regional-pack, the EU pack zones (`*.eu.oyatie.com`) MAY use a
  per-pack DNS provider where the regulatory pack mandates EU-resident
  control planes (e.g., a CSAP-equivalent EU regulator that prohibits
  US-hosted authoritative DNS for sovereign zones; this is rare but
  documented as a per-pack capability).
- **Anycast** — both providers run Anycast natively; queries route to
  the nearest POP based on BGP topology.
- **GeoDNS** — per-region health-checked records. A query for
  `app.oyatie.com` from a Berlin IP resolves to the EU cell's edge
  ingress IP; from a Seoul IP, to the KR cell's edge ingress IP.
- **Per-region health-checks** — Cloudflare DNS health-check + Route
  53 health-check both probe the per-cell edge ingress every 10 seconds
  from multiple POPs. A region's record is **withdrawn** when 3 of 4
  POP-probes fail consecutively (30 second detection window).
- **Failover-on-health-down** — when EU cell health-check fails,
  GeoDNS for EU clients fails over to **dr_cell** per ADR-0241 (not
  to US cell — privacy and latency reasons; an EU client failing over
  to a US cell would cross GDPR boundaries unless the tenant has
  explicit cross-region consent per ADR-0049 cross-region replication
  + residency).
- **TTLs** — DNS records have intentionally low TTLs (60 seconds for
  apex A/AAAA + ingress CNAMEs; 300 seconds for MX, SPF, DKIM, DMARC).
  Low TTL trades cache hit ratio for failover responsiveness.
- **DNSSEC** — apex zone is DNSSEC-signed with Cloudflare DNS's KSK +
  ZSK; KSK rotation annually; ZSK rotation quarterly. KSK held in
  oyatie's HSM (per ADR-0240 key-custody-BYOK pattern) once self-hosting transitions.
- **DNS-over-HTTPS / DNS-over-TLS** — apex zone supports DoH + DoT for
  client resolution privacy.

**Year 3+ (self-hosted migration target):**

- Self-host **PowerDNS Authoritative + PowerDNS Recursor** in dev-
  tools-cell + service-cells per pack. Anycast IP advertisement via
  oyatie-owned BGP advertisements (per D-12). Auth + recursive
  separated; auth NS only respond to queries for oyatie-managed
  zones; recursive NS only serve oyatie-internal resolvers.
- Continue using Cloudflare DNS + Route 53 as **secondary** for
  resilience (defence-in-depth against operator error in the self-
  hosted stack).
- Migration triggered by: (1) oyatie hits ~30 POP-class anycast
  locations; (2) DNS query volume exceeds 100k QPS sustained;
  (3) ADR-0211 self-hosting criteria met (controllable, predictable,
  cost-justified).

**Bind/PowerDNS over Cilium for self-hosted.** When self-hosted, the
DNS µservice runs as K8s Pods in dev-tools-cell + service-cells.
Pods are exposed via Cilium L4 service-of-type-LoadBalancer with
external traffic policy = Local (preserves source IP for DDoS
attribution). Anycast IP is advertised via BGP from cell edge
routers; Cilium's BGP control plane (per Cilium 1.16+ docs 2024)
manages the advertisement, OR a dedicated BIRD / FRRouting daemon
runs on edge router VMs.

### D-2. Edge layer — Cloudflare Workers initially, self-host Pingora Year 3+

The edge layer (planetary CDN + DDoS absorption + WAF + bot mitigation
+ TLS termination) operates as **Cloudflare Workers (V8 isolates) at
~300 POPs globally** for Year 1-2; **self-hosted Pingora POPs at
oyatie-owned datacentres** by Year 3+.

**Year 1-2 (Cloudflare-hosted):**

- **CDN** — Cloudflare CDN for static asset caching (consumer brand
  surfaces, marketing site, documentation site). Per-asset cache TTLs
  per `Cache-Control` response header authored by the origin µservice.
  Per-tenant cache key includes tenant_id for tenant-scoped assets
  (per ADR-0244 tenant-as-universal-scoping-primitive).
- **DDoS absorption** — Cloudflare's built-in DDoS protection (L3/L4
  + L7) absorbs volumetric attacks at the edge. Per-zone rate limits
  configured. Argo Smart Routing (Cloudflare premium feature) for
  origin-to-edge optimisation when latency premium justifies it.
- **WAF (Web Application Firewall)** — Cloudflare WAF with OWASP Core
  Rule Set (CRS) enabled + Cloudflare Managed Ruleset enabled.
  Per-zone custom rules for oyatie-specific patterns (e.g., block
  unauthenticated requests to `/api/internal/*`; rate-limit per-
  tenant Cedar gate calls; block IPs in OFAC sanctioned countries
  for non-sanctioned-tenant traffic).
- **Alternative WAF (open-source path):** **Coraza** (open-source,
  Apache 2.0 license, OWASP CRS-compatible) is the alternative WAF
  for the self-hosted Pingora path. Coraza is a Go-based WAF library
  that integrates with Caddy / Traefik / Pingora-via-plugin. Until
  Pingora migration completes, Coraza runs at the per-cell ingress
  Envoy as a Lua filter (Coraza's `coraza-envoy-wasm` plugin) for
  defence in depth — even if Cloudflare's WAF mis-classifies, Coraza
  has a second pass.
- **Bot mitigation** — Cloudflare Bot Management (Super Bot Fight
  Mode); per-zone configuration. Combined with hCaptcha for human-
  verification challenges where bot suspicion is high but not
  conclusive.
- **TLS termination** — TLS 1.3 only; HTTP/3 (QUIC) enabled per-zone;
  HTTP/2 + HTTP/1.1 fallback; per ADR-0240 key-custody-BYOK pattern, oyatie
  certificates use cosign-attested Let's Encrypt + internal CA chain
  authored by the policy-engine µservice and synced to Cloudflare
  via Cloudflare API.
- **Edge does NOT run in K8s.** V8 isolates at POP density too fine
  for K8s — see §Context "Why edge is NOT in K8s."

**Year 3+ (self-hosted Pingora):**

- **Pingora** — Cloudflare's Rust-based proxy, open-sourced February
  2024 (CC0/MIT license; full open-source release at
  `github.com/cloudflare/pingora`). Designed for HTTP/2 + HTTP/3 +
  TLS 1.3 at planetary scale; powers Cloudflare's own edge.
- **Per-POP Pingora deployment** — Pingora runs as a long-lived
  process (NOT in K8s, NOT as Pods) on edge-router VMs. Each POP runs
  N Pingora workers (per-core; CPU-pinned for cache locality).
  Pingora's hot-restart-without-dropping-connections capability
  matches Cloudflare's production deployment pattern.
- **Per-POP V8 isolates (optional)** — for tenant-authored edge
  workers (e.g., a tenant's custom request transformation in the
  hot path), oyatie deploys an **isolate-cloud** layer atop Pingora.
  This matches Cloudflare Workers' shape. Year 3+ scope.
- **Per-POP capacity** — ~30 POPs initially (matching Stripe's
  initial edge footprint); ~100 POPs by Year 5; ~300 POPs by Year 7
  (matching Cloudflare's current footprint).
- **Migration timeline** — Year 3 begins migration of one POP per
  quarter; transition validation requires zero observable client-side
  regression for 30 consecutive days before each subsequent POP
  migration.

### D-3. TLS termination — per-cell ingress at cell edge gateway

Per-cell **ingress termination** at the cell edge gateway.

- **Per-cell ingress gateway:** **Envoy** (as Istio Ambient ingress
  gateway per ADR-0148) **OR** Cilium ingress (where Cilium 1.16+'s
  Gateway API conformance is sufficient for the cell's traffic
  patterns).
- **Decision:** Envoy primary for cells handling regulatory packs
  (richer L7 surface for response envelope mutation per ADR-0148);
  Cilium ingress for cells handling simple east-west fan-in (no L7
  policy needed).
- **TLS 1.3 only.** TLS 1.0, 1.1, 1.2 disabled at every termination
  point. Per ADR-0044 strict-mTLS posture.
- **Cipher suites** — per Mozilla SSL Configuration Generator
  "modern" profile, TLS 1.3 with the three NIST-recommended cipher
  suites: `TLS_AES_256_GCM_SHA384`, `TLS_CHACHA20_POLY1305_SHA256`,
  `TLS_AES_128_GCM_SHA256`. ChaCha20-Poly1305 prioritised for mobile
  clients (faster on ARM CPUs without AES-NI).
- **Certificates:**
  - **Public-facing certificates** — per-tenant Let's Encrypt
    certificates (via Cloudflare DNS-01 challenge); auto-renewed
    every 60 days; cosign-attested at issuance time per
    ADR-0223 (oya git canonical) signed-config pattern, and the
    cosign-attestation lives in the policy-engine µservice as a
    signed Cedar fragment referencing the cert chain.
  - **Internal certificates** — internal CA chain rooted in the
    org root key (per ADR-0242 §D-5 bootstrap sequence; the root key
    lives in a tier-0 HSM / YubiKey HSM cluster). Internal CA issues
    intermediate CAs per cell; per-µservice SVIDs (SPIFFE) are
    issued by SPIRE at the leaf (D-7).
- **Certificate management** — per ADR-0223 oya git canonical, the
  policy-engine µservice stores cert chains as signed config; cert
  issuance + rotation + revocation runs as Workflow Engine durable
  saga (per ADR-0241 saga pattern + per ADR-0252 idempotency-keys);
  emergency revocation propagates through CRL + OCSP-stapling within
  5 minutes.
- **OCSP stapling** — enabled at every ingress; freshness window
  capped at 24h; Must-Staple TLS extension set on issued certs.

### D-4. Post-quantum hybrid TLS — ML-KEM-768 + X25519 hybrid KEX, Year 2 deployment

The platform deploys **post-quantum hybrid key exchange** per the NIST
PQC standardisation (FIPS 203 "Module-Lattice-Based Key-Encapsulation
Mechanism," August 2024).

- **Algorithm:** ML-KEM-768 (formerly known as Kyber-768) hybrid with
  X25519. The hybrid combines a classical KEX (X25519, Curve25519-
  based) with a post-quantum KEX (ML-KEM-768), per IETF Internet-
  Draft `draft-ietf-tls-hybrid-design`. The shared secret is the
  concatenation of both component secrets, fed through HKDF; an
  attacker must break BOTH components to recover the session key.
- **Deployment target:** Year 2 (2027-Q2 to 2028-Q1).
  - **2026-Q4 to 2027-Q1:** Cloudflare edge already supports
    X25519MLKEM768 (per Cloudflare blog 2024 "Post-quantum to the
    people"). Oyatie's Cloudflare zones inherit this for free.
  - **2027-Q2:** Per-cell ingress Envoy upgraded to BoringSSL or
    OpenSSL 3.x with `liboqs` provider; PQ hybrid KEX advertised on
    public endpoints.
  - **2027-Q3:** AWS s2n-tls Pods (where present per ADR-0240
    pack matrix) on PQ hybrid; AWS supports `kyber-tls13` hybrid
    suite since 2024.
  - **2027-Q4:** Inter-cell mTLS (SPIRE-issued SVIDs) gains PQ
    hybrid KEX; SPIRE 1.10+ supports `liboqs` PQ certificate
    extensions via OpenSSL provider.
  - **2028-Q1:** Service-to-service east-west mesh fully on PQ
    hybrid; legacy classical-only KEX disabled at the cipher-suite
    config.
- **Defence against "harvest-now-decrypt-later" attacks.** The threat
  model: a state-level adversary records TLS-1.3-classical-encrypted
  traffic in 2026 and decrypts in 2032+ when a CRQC (cryptographically
  relevant quantum computer) is hypothesised to exist. Hybrid KEX
  raises the bar: the recorded traffic remains secure as long as
  EITHER the classical or the PQ component is unbroken. Best
  estimate: CRQC arrival is 2030-2040 (per NIST + NSA CNSA 2.0
  guidance); the 2-year deployment window opens 4-12 years of
  protection before the boundary.
- **Algorithm agility.** The config-file format names cipher suites
  symbolically; replacing ML-KEM-768 with a future PQ algorithm
  (FIPS 204 ML-DSA for signatures + future PQC round 4 candidates if
  ML-KEM is broken) is a config change, not a code change.
- **Digital signatures (later):** ML-DSA (FIPS 204 "Module-Lattice-
  Based Digital Signature Algorithm," August 2024) for code signing
  and audit-chain seals targeted 2028-Q4 (Year 3). Ed25519-classical
  remains primary; ML-DSA hybrid added defence-in-depth in Year 3.

### D-5. HTTP/3 (QUIC) client-side everywhere

Every public-facing endpoint supports **HTTP/3 (QUIC)** per RFC 9114
+ RFC 9000.

- **Why HTTP/3.** RFC 9114 (HTTP/3) ratified 2022; RFC 9000 (QUIC)
  ratified 2021. Benefits over HTTP/2:
  - 0-RTT session resumption (encrypted in TLS 1.3) saves ~100ms on
    repeat connections.
  - Stream multiplexing without head-of-line blocking (TCP-level
    HoL blocking is solved by per-stream packetisation).
  - Connection migration across network change (mobile WiFi ↔ LTE
    without reconnect).
  - Improved congestion control (BBR + Cubic optionals; per-stream
    flow control).
- **Deployment.**
  - **Edge:** Cloudflare zones have HTTP/3 enabled by default since
    2020 (universal HTTP/3 rollout).
  - **Per-cell ingress:** Envoy 1.30+ supports HTTP/3 via the QUIC
    listener filter chain; enabled at every public ingress.
- **Fallback order:** HTTP/3 (preferred) → HTTP/2 (Alt-Svc fallback)
  → HTTP/1.1 (legacy clients only; deprecated 2028).
- **Service-to-service stays HTTP/2.** Internal east-west mesh traffic
  uses **gRPC over HTTP/2** for the foreseeable future. Reasons:
  - HTTP/2 is mature for east-west; performance differential vs
    HTTP/3 is < 5% for in-cell traffic where TCP HoL blocking is
    rare.
  - QUIC over east-west routes encrypts UDP, which interacts poorly
    with kernel-bypass eBPF observability (Cilium Hubble's flow
    records are TCP-aware by default; UDP flow records less
    informative). Per ADR-0148 Tier 1 = Cilium L4 + Hubble flow obs.
  - gRPC-over-QUIC is an emerging spec but ecosystem support is
    behind gRPC-over-HTTP/2.
- **WebTransport** for novel use cases (e.g., realtime collaborative
  editing on the Workflow Studio canvas; voice/video signalling).
  WebTransport runs over HTTP/3; ingress and edge layers support it
  natively.

### D-6. Cilium ambient service mesh inside cells (per ADR-0148)

The in-cell service mesh is per ADR-0148: **Cilium 1.16 LTS at L3/L4
+ Istio Ambient 1.24 LTS at L7 (zero feature overlap; layered)**.

This ADR adds the cross-cell + cross-provider extensions:

- **Cilium ClusterMesh** for cross-cell L4 topology within the same
  provider (e.g., two AWS-Frankfurt cells in the EU pack). ClusterMesh
  trust bundles federate; per-cell SPIRE identity bridges to
  per-cluster Cilium identity.
- **Istio Ambient cross-cluster mesh** via SPIFFE federation (D-7) for
  cross-cell L7. Waypoint resources may declare cross-cell endpoint
  selectors via `ServiceEntry` resources; mTLS termination at the
  receiving cell's waypoint per cell-local Cedar policy.
- **eBPF-based, sidecarless** — Cilium's L3/L4 dataplane runs in the
  kernel; Istio Ambient's L7 dataplane runs as per-namespace waypoint
  Pods + per-node ztunnel (Rust). Per-µservice waypoint enrolment is
  opt-in per ADR-0148 `mesh_layering.ambient_waypoint: true|false`.
- **Cedar PDP wiring** — Istio Ambient waypoint `ext_authz` filter
  calls policy-engine Cedar PDP over gRPC for every L7-policed request.
  Per ADR-0148 + ADR-0183 + ADR-0243.

### D-7. SPIFFE workload identity — every Pod has a SPIFFE SVID, rotated hourly

Every Pod has a **SPIFFE SVID** issued by **SPIRE 1.10+**.

- **SPIFFE / SPIRE.** SPIFFE (Secure Production Identity Framework
  For Everyone) is the CNCF graduated standard (graduated 2022) for
  workload identity. SPIRE is the reference implementation.
- **SVID issuance.**
  - SPIRE Server per cell (with per-cell HSM-backed signing key).
  - SPIRE Agent per node (DaemonSet); attests workloads via
    Kubernetes attestor (verifies Pod's service account + namespace
    + sa-token); issues X.509 SVID with SPIFFE-ID URI SAN.
  - SVID format: SPIFFE-ID `spiffe://<trust-domain>/<path>`.
    Oyatie trust domain pattern:
    `spiffe://<cell-id>.<pack-id>.oyatie/ns/<namespace>/sa/<service-account>`.
    Example: `spiffe://cell-frankfurt-1.eu-pack.oyatie/ns/policy-engine/sa/cedar-pdp`.
- **Per-tenant scope.** SPIFFE-ID encodes the µservice's identity;
  the tenant_id is carried in the request body / metadata as a
  separate claim (Cedar principal `Tenant::"acme-corp"`). SPIFFE-ID
  identifies the workload; the request authentication identifies the
  acting tenant.
- **Rotation.** SVIDs rotate every **1 hour** (SPIRE default). Short
  rotation interval limits exposure if an SVID is exfiltrated; the
  exfiltrated SVID is useless after the 1-hour TTL expires.
- **Cedar gate consumes SPIFFE-ID as principal claim.** Per ADR-0243,
  every Cedar policy evaluation receives the caller's SPIFFE-ID as
  the workload principal claim. Cedar fragments may permit
  `principal in WorkloadGroup::"trusted-internal"` where
  `WorkloadGroup` membership is defined by SPIFFE-ID glob (e.g.,
  `spiffe://*.oyatie/ns/policy-engine/sa/*`).
- **SPIRE federation across cells.** Per-cell SPIRE servers federate
  trust bundles via the **SPIFFE Federation API** (per SPIFFE spec
  v0.6). Cross-cell calls validate the foreign cell's trust bundle
  against the federated trust list. Cell removal triggers federation
  bundle revocation; cross-cell calls from a removed cell fail at
  the mTLS handshake.

### D-8. Inter-cell traffic — cross-cell async slow path per ADR-0248

Inter-cell traffic follows ADR-0248 (Amazon-shape cellular
architecture): **mostly async; cross-cell is the slow path; in-cell
is the fast path.**

- **Transport.** mTLS via **SPIFFE-federated SVIDs**. The receiving
  cell's waypoint terminates mTLS using the federated trust bundle.
- **Path.** Cross-cell traffic crosses provider backbone (e.g., AWS
  Direct between regions; NHN private link between
  Pangyo zones) OR public internet (with WireGuard tunnel — see D-9).
- **Cedar gate per call.** Per ADR-0243 + ADR-0148 Cedar `ext_authz`
  wiring, the receiving waypoint evaluates Cedar policy for the
  cross-cell call. Per-call policy fragments may differ from
  intra-cell policy (e.g., "permit cross-cell read of ontology
  projection IFF tenant is the same and data class is not
  `SOVEREIGN_RESTRICTED`").
- **Failure modes.** Cross-cell call failure is **opaque to the
  caller**: the caller retries via durable saga (Workflow Engine) or
  fails fast with a typed error. No silent degradation. Per
  `feedback_no_silent_regression`.
- **Idempotency.** Cross-cell write calls require idempotency-key
  per ADR-0252; cross-cell read calls are at-most-once read with
  caller-side retry budget.
- **Per-tenant trust matrix.** A tenant pinned to EU pack
  (`tenant.home_cell = cell-frankfurt-1.eu-pack`) calling its DR
  cell (`tenant.dr_cell = cell-dublin-1.eu-pack`) is cleared by
  Cedar; calling US-pack (`cell-northern-virginia-1.us-pack`) is
  denied unless the tenant has explicit cross-pack residency
  exemption (per ADR-0049 + ADR-0240).

### D-9. Cross-provider traffic — WireGuard tunnel, per-pair Cedar permits

Cross-provider traffic (e.g., AWS-Frankfurt → NHN-Cloud-Pangyo) per
ADR-0240 sovereign-cloud overlay uses an **inter-provider mesh tunnel**.

- **Transport.** **WireGuard** (preferred; kernel-mode on Linux; key
  rotation via SPIRE-issued X.509 → WireGuard preshared key bridge)
  OR **Tailscale** (managed WireGuard; for cells that need control-
  plane managed key distribution without operating a WireGuard
  control plane).
- **Topology.** Per-provider-pair tunnels (e.g., AWS-eu-pack to
  NHN-kr-pack); NOT full mesh (full mesh is O(N²) and operationally
  brittle at >10 providers). Hub-and-spoke for tenants needing
  multi-provider routing; the hub is operated by `oyatie.platform-ops`
  in a dedicated tier-2 cell.
- **Cedar gate per call.** Per-provider-pair Cedar permits required;
  default-deny per ADR-0240. A tenant in EU pack calling a service
  in KR pack must have explicit Cedar permit declaring the cross-
  provider edge (e.g., `permit (principal in TenantGroup::"eu-with-
  kr-extension", action == "Read", resource in Cell::"cell-pangyo-1")`).
- **Sovereign-data-class denied egress.** Per ADR-0240, data with
  class `SOVEREIGN_RESTRICTED_EU` or `SOVEREIGN_RESTRICTED_KR` is
  **never** egressed across provider boundaries. The Cedar fragment
  for cross-provider permits explicitly forbids resources tagged
  with sovereign-restricted data classes.
- **Encryption.** WireGuard encrypts at the IP layer (Curve25519 +
  ChaCha20-Poly1305 + BLAKE2s + SipHash24); mTLS over WireGuard
  provides defence in depth.

### D-10. Network policy at K8s layer — Cilium default-deny egress

In-cell network policy at the K8s layer is **Cilium NetworkPolicy +
CiliumClusterwideNetworkPolicy** with **default-deny egress**.

- **Default-deny egress.** Every namespace has a default-deny egress
  policy; explicit allow lists per µservice per ADR-0148 §Operational.
- **Per-µservice allow list.** Each µservice's
  `iac/helm/<ms>/templates/ciliumnetworkpolicy.yaml` declares the L4
  IDENTITY rules: allowed outbound sibling-µservices, allowed
  outbound FQDNs (e.g., `api.stripe.com` for billing-µservice),
  allowed CIDRs (e.g., per-cell IPAM ranges).
- **FQDN-aware egress.** Cilium 1.16's FQDN policy (DNS-aware) resolves
  the FQDN at policy-evaluation time; the allow list updates dynamically
  as the FQDN's A/AAAA records change.
- **Cilium identity** — per-Pod identity derived from labels; policy
  evaluates on identity equivalence (not IP), so Pod restart with
  new IP doesn't break the policy.
- **CiliumClusterwideNetworkPolicy** for cluster-spanning rules
  (e.g., block all egress to internal RFC1918 ranges from the
  `external-edge` namespace).

### D-11. Load balancing — per-cell L4 (Cilium) + L7 Envoy at ingress

Load balancing operates at multiple tiers:

- **L4 (transport-layer) load balancing inside cell:** Cilium's
  eBPF-based service load balancer. Per-service ClusterIP load
  balancing; consistent-hash mode optional for sticky sessions.
- **L7 (application-layer) load balancing at ingress:** Envoy as
  Istio Ambient ingress gateway (per ADR-0148). Per-route load-
  balancing policy: round-robin, least-request, ring-hash, Maglev
  consistent-hash, random.
- **Per-µservice service mesh load balancing:** Cilium ambient
  service-to-service load balancing at L4; waypoint Envoy L7 load
  balancing when the µservice has enrolled a waypoint.
- **Connection draining.** Per Envoy / Cilium, `terminationDrainDuration`
  default 30s; in-flight requests complete before the Pod terminates;
  new connections route to other replicas.
- **Health checks.** Per-service active health checks (Envoy's HTTP
  health-check filter); per-Pod readiness probe per K8s; failed Pods
  removed from the load-balancing pool within 5 seconds.
- **Outlier detection.** Envoy outlier-detection filter ejects Pods
  with 5xx-rate > 50% from the load-balancing pool for 30s;
  re-includes after 30s of healthy probes.

### D-12. BGP / IP advertisement — cloud-provider BGP initially, self-managed Year 5+

IP advertisement and BGP:

- **Year 1-3 (cloud-provider BGP):** Cloud providers (AWS, Azure, GCP,
  NHN, KT, NaverCloud, OCI) advertise per-cell ingress IPs via the
  provider's own BGP infrastructure. Oyatie does not operate AS
  numbers or own BGP peers in this phase.
- **Year 4-5 (transit-provider BGP):** Oyatie acquires an Autonomous
  System Number (ASN) from RIPE NCC + ARIN + APNIC + KRNIC. IP space
  acquired (IPv4 /24 minimum for visible BGP; IPv6 /48 standard).
  Oyatie operates BGP peers with Tier-1 transit providers (Cogent,
  Lumen, NTT, Telia) at Internet Exchange Points (DE-CIX Frankfurt,
  AMS-IX, LINX London, KINX Seoul, JPIX Tokyo, Equinix Ashburn).
  IP space is advertised via Anycast across oyatie POPs.
- **Year 5+ (self-managed BGP at scale):** When oyatie owns
  datacentres (per ADR-0211 in-house preference at scale), oyatie's
  network ops team operates BGP routers (Arista, Juniper, Cisco) and
  software BGP daemons (BIRD, FRRouting) for fine-grained route
  control. RPKI signing for route origin authentication; route
  filtering against BOGON lists; per-peer prefix limits.
- **BGPsec or RPKI ROA** — route-origin attestation required for all
  advertised prefixes. Per RFC 6810 + RFC 8205.
- **DDoS-via-BGP-blackhole** — coordinate with transit providers for
  RTBH (remotely-triggered blackhole) on volumetric DDoS that exceeds
  edge POP absorption capacity. RFC 5635 + RFC 7999.

### D-13. Multi-region routing — GeoDNS routes client to home_cell

GeoDNS routes clients per ADR-0241 DR + ADR-0049 residency.

- **Default:** GeoDNS routes the client to the cell housing the
  tenant's **home_cell** (per ADR-0244 tenant model). For a Berlin
  tenant whose home_cell is `cell-frankfurt-1.eu-pack`, the GeoDNS
  resolution for that tenant's apex hostname (e.g.,
  `acme-corp.oyatie.com`) returns the EU cell ingress IP.
- **Tenant-aware DNS — TXT-based discovery.** Per-tenant apex
  hostnames are dynamic. The discovery flow:
  1. Client queries `_oyatie-cell.acme-corp.oyatie.com` (TXT record)
     to learn the tenant's preferred cell (cached by client SDK).
  2. Client connects to the cell's apex ingress
     (`cell-frankfurt-1.eu.oyatie.com`).
  3. TLS SNI carries the tenant subdomain
     (`acme-corp.oyatie.com`) for cert selection.
- **Failover to dr_cell.** When the home_cell health-check fails,
  GeoDNS for the tenant's apex fails over to the tenant's `dr_cell`
  per ADR-0241. The dr_cell is **same-pack** when possible (e.g.,
  EU tenant fails over to another EU cell, not to US cell).
- **Intelligent fallback.** Example: EU client whose home_cell is
  `cell-frankfurt-1.eu-pack` AND dr_cell is `cell-dublin-1.eu-pack`.
  When Frankfurt cell is down:
  - DNS resolution for `acme-corp.oyatie.com` from EU client →
    `cell-dublin-1.eu.oyatie.com` (EU DR cell — preferred).
  - DNS resolution for `acme-corp.oyatie.com` from US client AT THE
    SAME TIME (e.g., a US visitor of the EU tenant's public site) →
    still `cell-dublin-1.eu.oyatie.com` (EU residency for EU
    tenant; US client tolerates trans-Atlantic latency).
  - DNS resolution NEVER routes EU tenant data to US cell unless
    the tenant has explicit cross-region residency exemption.
- **Geographic exclusion.** Per ADR-0240 sovereign-cloud-per-regional-
  pack, certain tenants (e.g., regulated EU financial-services with
  ECB residency requirements) have GeoDNS configured to NEVER
  resolve to non-EU cells; if both EU cells fail, the tenant is in a
  **planned outage** rather than a residency violation. The tenant's
  business continuity contract documents this tradeoff.

### D-14. HTTP API surfaces — REST 3.2 + gRPC + AsyncAPI 3.1

Per oyatie canonical API spec authority:

- **REST per OpenAPI 3.2.0.** Public + tenant-facing read/write APIs.
  Per oyatie's existing OpenAPI spec authority. Versioning per RFC
  9457 (Problem Details for HTTP APIs) + standard `Accept-Version`
  header pattern. Cursor pagination per ADR-0150. Idempotency keys
  per ADR-0252.
- **Historical rejected context:** GraphQL Federation v2 via a BFF tier was considered for rich UI
  surfaces. ADR-0565 removed that leg; rich-UI read aggregation is served by REST/gRPC composition.
- **gRPC** for high-throughput internal calls + low-latency RPCs
  between µservices. Per ADR-0145 direct-sibling-gRPC permitted.
  Protocol Buffers v3; per-µservice `contracts/proto/*.proto` files
  authored against `buf` linter rules; backward-compatibility
  enforced via `buf breaking` CI lane.
- **AsyncAPI 3.1.0** for event-driven APIs. Per-µservice
  `contracts/asyncapi/*.yaml` files. Event channels backed by NATS
  (per ADR-0149 boundary) for fan-out + by Kafka for ordered
  durable streams + by audit-chain for sealed evidence streams.
- **No raw HTTP without one of the above.** Every public + tenant-
  facing surface MUST declare its contract (OpenAPI / Proto /
  AsyncAPI). GraphQL SDL is historical rejected context under ADR-0565. CI lane `oya-check-api-contract-presence`
  enforces.
- **No "internal-only" surfaces lack contracts.** Per ADR-0242
  oyatie-is-a-tenant doctrine; all surfaces have contracts.

### D-15. WebSocket + SSE — SSE primary, WebSocket for bidirectional

Realtime push:

- **Server-Sent Events (SSE) — primary for one-way push.** Used for:
  - LLM token streams (per ADR-0255 Intelligence substrate);
  - Notification streams (per `microservices/notifications/`);
  - Audit-chain seal confirmations.
  SSE is simpler than WebSocket (HTTP-native; auto-reconnect; no
  framing protocol); per-tenant rate limits at edge.
- **WebSocket — for bidirectional.** Used for:
  - Collaborative editing (Workflow Studio canvas; ADR-0142 CRDT
    portability trait already established);
  - Presence indicators (who's online; what's the cursor position);
  - Voice/video signalling (WebRTC SDP exchange);
  - Realtime dashboards with bi-directional control.
  Per-tenant + per-connection rate limits at edge; per-µservice
  fairness budgets.
- **Per-cell push gateway.** A per-cell push gateway µservice
  (`microservices/push-gateway/`) manages WebSocket + SSE
  connections. Tenant is **pinned to home-cell gateway** for the
  duration of a session (sticky-session via cookie + Anycast IP
  preservation); on failover, the client SDK re-establishes against
  dr_cell.
- **WebTransport (D-5)** for use cases where HTTP/3-native
  bidirectional streaming is preferred over WebSocket (e.g.,
  multiplayer collaboration with strict latency requirements).

### D-16. Webhook outbound — signed delivery, durable retry, idempotency

Outbound webhooks (oyatie pushing events to tenants' systems):

- **Signed delivery (HMAC + Ed25519 dual-signed).**
  - **HMAC-SHA256** of payload using per-tenant shared secret in
    the `X-Oyatie-Signature-HMAC` header (Stripe-style; backward
    compatible with the most common webhook verification pattern).
  - **Ed25519 signature** of payload using oyatie's per-region
    Ed25519 signing key (rotated quarterly; public key published
    at well-known URI `https://oyatie.com/.well-known/webhook-
    signing-keys.json`); signature in `X-Oyatie-Signature-Ed25519`
    header (Slack-style + GitHub-style modern pattern).
  - Tenant SHOULD verify both signatures; tenant MAY verify only
    HMAC during incremental migration to Ed25519 verification.
- **Retry via Workflow Engine durable saga.** Webhook delivery is a
  durable workflow (per ADR-0145 Workflow Engine pattern). Retry
  schedule: 1m, 5m, 30m, 2h, 6h, 24h. After 24h of failures, the
  webhook is marked `dead` and the tenant is notified via
  alternative channel (email + in-product alert).
- **Idempotency-key required.** Per ADR-0252 idempotency-keys
  canonical; the receiving side (tenant) MUST deduplicate by
  `X-Oyatie-Idempotency-Key`. Oyatie's send-side ensures uniqueness
  across retries (the same key for the same event across all retry
  attempts).
- **Per-tenant subscription per event type.** Tenants subscribe to
  events at granular event-type level (e.g.,
  `workflow.run.completed`, `tenant.user.created`,
  `intelligence.batch.completed`); Cedar gate evaluates whether the
  tenant has permission to subscribe + whether the subscription is
  within per-tenant rate limit.
- **Per-tenant outbound rate limit.** Default 10 webhooks per second
  per tenant; soft cap; configurable per-tenant in tenancy config.
- **Tenant-side TLS verification.** Oyatie webhooks require the
  tenant's webhook URL to have a valid TLS cert (Let's Encrypt /
  ZeroSSL / commercial CA); plain HTTP tenant URLs accepted only
  for tenants in oyatie.dev sandbox sub-tenants.

### D-17. Pingora migration path — Year 3+ from Cloudflare to self-hosted

The full Pingora migration path:

- **Year 1-2: Cloudflare-hosted.** Standard zone configuration;
  Cloudflare Workers for Cedar-bound edge logic (e.g., per-tenant
  rate limits via Workers Durable Objects); Cloudflare's own DDoS
  + WAF + Bot Management.
- **Year 3-Q1: First oyatie-owned POP.** Acquire colocation space at
  a Tier-1 IX (Equinix Ashburn or DE-CIX Frankfurt). Deploy
  Pingora on bare-metal Arm or AMD-EPYC servers. ASN advertised
  via BGP (D-12). One pilot tenant migrated; verify zero observable
  client regression for 30 days.
- **Year 3-Q2 to Year 3-Q4: 10 POPs deployed.** Pace of one POP per
  ~3-4 weeks. Each POP onboarding includes:
  1. Colocation provisioning (rack + power + cross-connect);
  2. Bare-metal install (Talos / Debian + containerd + Cilium for
     management plane; Pingora on host for data plane);
  3. BGP peering with local IX peers;
  4. SPIRE Server federation;
  5. Pingora hot-restart deployment;
  6. Smoke testing + 30-day soak.
- **Year 4: 30 POPs deployed.** Coverage matches Stripe's edge
  footprint.
- **Year 5: 100 POPs deployed.** Coverage matches Fastly's edge
  footprint.
- **Year 6-7: 300 POPs deployed.** Coverage matches Cloudflare's
  current footprint.
- **Cloudflare retained as secondary edge through Year 5.** Defence
  in depth against an oyatie-edge bug that takes down all oyatie-
  owned POPs simultaneously. Cloudflare-hosted edge runs in
  passive-active mode; GeoDNS prefers oyatie-owned POPs but fails
  over to Cloudflare POPs on health-check failure.
- **Pingora deployment shape.** Per CCloudflare engineering blog 2022
  + 2024:
  - Single Pingora process per server (multi-threaded; one worker
    per CPU core; epoll-based async I/O).
  - Connection pooling for upstream connections (one pool per
    upstream service; LRU eviction).
  - Hot-restart preserves in-flight connections (Pingora's
    "graceful upgrade" feature).
  - Configuration as code (Pingora's YAML config compiled to
    binary; deployed via Flux Helm releases for K8s-managed POPs,
    via Ansible for bare-metal POPs).
- **Pingora extensions.**
  - Custom request transformation: Lua filters (via `pingora-proxy`
    Lua plugin) OR Rust extensions compiled into the Pingora binary.
  - Cedar PDP wiring at edge: Pingora calls policy-engine Cedar PDP
    over gRPC; per-tenant rate limits and per-edge-rule decisions
    evaluated at the POP.

### D-18. Self-hosted DNS migration path — Year 3+ PowerDNS in dev-tools-cell

The full self-hosted DNS migration:

- **Year 1-2: Cloudflare DNS + Route 53.** Dual-authoritative for
  resilience.
- **Year 3-Q1: PowerDNS staging.** PowerDNS Authoritative deployed
  in dev-tools-cell as K8s Pods (Helm chart from PowerDNS upstream).
  Zone files synced from Cloudflare DNS + Route 53 (one-way
  replication). Zero traffic routed to PowerDNS yet.
- **Year 3-Q2: PowerDNS in service cells.** Deploy PowerDNS in each
  service cell (one PowerDNS server per cell); each cell's PowerDNS
  serves the local cell's zone (e.g., `cell-frankfurt-1.eu.oyatie.com`
  is served from the Frankfurt cell's local PowerDNS).
- **Year 3-Q3: Anycast advertisement.** Each cell's PowerDNS gets an
  Anycast IP; the IP is advertised via BGP from the cell's edge
  routers (D-12). DNS query routing follows BGP topology.
- **Year 3-Q4: Authoritative cutover.** NS records for `oyatie.com`
  + `eu.oyatie.com` + `kr.oyatie.com` + ... cutover to oyatie-owned
  authoritative NS records. Cloudflare DNS + Route 53 remain as
  secondary for resilience.
- **Year 4-5: PowerDNS Recursor.** Internal resolution (for in-cell
  workloads resolving sibling-µservice FQDNs, external API hostnames)
  uses PowerDNS Recursor (per-cell). Recursor caches; DNSSEC
  validation enabled.
- **Auth + recursive separated.** Authoritative servers (which
  publish oyatie zones to the internet) are physically separated
  from recursive servers (which resolve queries on behalf of in-cell
  workloads). This separation is standard DNS practice (per RFC 8499
  + BCP 219).

## Alternatives considered

### Alt-1. Cloud-provider load balancer only (no edge POPs)

Route clients directly to per-cell ingress via cloud-provider load
balancer (AWS ALB / GCP HTTPS LB / Azure Front Door); no separate
edge layer. Cloud provider's built-in DDoS + WAF provides edge
defence.

**Pros:**
- Zero edge-layer engineering cost.
- Cloud provider's native DDoS protection (AWS Shield, Cloud Armor,
  Azure DDoS Protection).
- One-vendor relationship simplifies billing + procurement.

**Cons:**
- **No edge presence at POP density.** AWS has 100+ regions/AZ
  combinations; Cloud Armor has ~100 POPs; Azure Front Door has
  ~150 POPs. Cloudflare has 300+; Akamai has 4,000+. Latency to
  Berlin client from US-East-1 is ~110ms; from Frankfurt-1 is ~10ms;
  from Cloudflare Berlin POP is ~3ms.
- **Vendor lock-in to cloud-provider edge.** Per ADR-0240 sovereign-
  cloud-per-regional-pack, a multi-pack tenant cannot ride a single-
  cloud edge.
- **Limited bot mitigation.** Cloud-provider WAFs are competent but
  Cloudflare's bot management is industry-leading; Akamai Bot
  Manager is the regulated-finance industry standard.
- **No edge compute.** Cloud-provider load balancers don't run V8
  isolates or Pingora-equivalent. Edge-bound logic (per-tenant
  rate limits, per-edge Cedar fragments) cannot live at the cloud
  load balancer.

**Rejected** because the latency budget is unmeetable at cloud-
provider POP density, and edge compute is required for the
hyperscaler shape.

### Alt-2. Istio classic (sidecar) instead of Cilium ambient

Use Istio classic sidecar (per-Pod Envoy) for in-cell mesh; skip
Cilium ambient + Istio Ambient.

**Pros:**
- Largest production deployment (Istio classic deployed at most
  enterprise K8s installations); mature ecosystem.
- One mesh project; one operator skill.

**Cons:**
- Per-pod Envoy sidecar imposes ~2× CPU overhead on request path
  (per Istio Ambient benchmarks); ~30% additional memory per pod
  (50-200MB RAM × pods).
- Sidecar lifecycle race conditions on Pod startup; documented
  Pod-readiness vs sidecar-readiness ordering bugs.
- Conflicts with Cilium eBPF L4 dataplane (double-encrypt + double-
  policy paths).
- **Already rejected by ADR-0148.** Reasserting it contradicts the
  bundle's keystone position.

**Rejected** per ADR-0148 §Alternatives (b). This ADR inherits
ADR-0148's rejection.

### Alt-3. Linkerd instead of Istio Ambient

Use Linkerd (Rust proxy, simplest operational footprint) for L7 mesh
instead of Istio Ambient.

**Pros:**
- Simplest operational footprint of any L7 mesh.
- Rust proxy aligns with ADR-0120 + ADR-0211 in-house preference.
- Strong default tracing + metrics.

**Cons:**
- Smaller AuthorizationPolicy ecosystem than Istio.
- No first-class `ext_authz` hook for Cedar PDP wiring.
- Smaller community + ecosystem.
- No waypoint-equivalent per-namespace L7 boundary.
- **Already rejected by ADR-0148.** Reasserting contradicts the
  bundle's keystone position.

**Rejected** per ADR-0148 §Alternatives (d).

### Alt-4. AWS App Mesh / Azure equivalent (vendor mesh)

Use AWS App Mesh (managed Envoy on EKS) or Azure Open Service Mesh
(managed Envoy on AKS) instead of CNCF-graduated standards.

**Pros:**
- Managed control plane on respective hyperscaler; lower ops cost.
- Native integration with cloud-provider IAM + observability.

**Cons:**
- **Vendor lock-in.** AWS App Mesh is AWS-specific (violates the
  multi-cloud + on-prem portability invariant per ADR-0240 +
  ADR-0121); Azure OSM is Azure-specific.
- Cannot run in on-prem cells (ADR-0121 KR primary cell on
  kubeadm).
- Per-pack provider matrix (ADR-0240) requires mesh portable across
  providers.

**Rejected** because oyatie ships in EU + KR + on-prem cells where
neither AWS nor Azure is the default provider.

### Alt-5. CHOSEN: Cilium ambient + Cloudflare→Pingora at edge + SPIFFE/SPIRE + TLS 1.3 + PQ hybrid Year 2 + HTTP/3 client-side

The selected alternative, fully specified in §Decision.

**Pros:**
- **Matches every named hyperscaler reference** (Cloudflare,
  Google, AWS, Microsoft, Apple, Stripe).
- **Edge-first latency budget meetable.** Cloudflare 300 POPs in
  Year 1-2; oyatie 30+ POPs by Year 4 via Pingora.
- **Vendor-neutral.** Cilium + Istio Ambient + SPIFFE + Cedar
  open standards; CNCF graduated.
- **Post-quantum ready.** Year 2 deployment window before CRQC
  arrival.
- **HTTP/3 + WebTransport future-proof.** Modern protocols at every
  client-facing endpoint.
- **Self-hosting path tested.** Pingora open-sourced 2024 (Rust);
  PowerDNS battle-tested for 25+ years; oyatie can self-host with
  predictable engineering.
- **Inter-cell + cross-provider mesh defined.** Closes the trust
  boundary gap that ADR-0148 deferred.
- **Aligned with bundle keystones.** ADR-0240 sovereign cloud +
  ADR-0241 DR + ADR-0242 oyatie-is-a-tenant + ADR-0243 Cedar + 
  ADR-0244 tenant + ADR-0248 cellular all assume this network
  shape.

**Cons:**
- **Two-vendor + two-self-hosted timeline.** Cloudflare + AWS
  (Route 53) in Year 1-2; Pingora + PowerDNS layered in Year 3+.
  Operational complexity grows during transition; mitigated by
  per-component deployment cadence (one POP per ~3-4 weeks).
- **Post-quantum hybrid not yet universally interoperable.** Older
  TLS clients (TLS 1.2-only, common in IoT) cannot negotiate PQ
  hybrid. Mitigation: classical + hybrid offered side-by-side until
  2028-Q4; legacy disabled thereafter.
- **Edge-side Cedar policy duplication.** Per-tenant rate limits +
  per-edge Cedar fragments evaluated both at edge (Cloudflare
  Worker / Pingora-via-gRPC-to-PDP) and at cell ingress. Mitigation:
  one Cedar source-of-truth; policy compiler emits both edge + cell
  artifacts.

**Accepted** as the foundational keystone for the network topology.

## Consequences

### Positive

1. **Planetary latency budgets modeled.** Edge POPs within ~30ms of
   every major population centre. Per-stage modeled p99 first-byte
   budgets (evidence: docs/performance-budgets/edge-first-byte-50ms-p99.md):
   - Scenario A (warm conn, HTTP/3 0-RTT, edge cache hit): ~9ms p99
     [P5..P95: 5ms–15ms] — well within 50ms.
   - Scenario B (warm conn, HTTP/3 1-RTT, dynamic request, same-region
     cell): ~51ms p99 [P5..P95: 40ms–75ms] — meets 50ms budget within
     measurement margin; conservative claim ≤60ms p99.
   - Scenario C (cold conn, new client, cross-region cell): ~207ms p99
     [P5..P95: 150ms–250ms] — uses relaxed 200ms budget.
   Per-cell ingress (steady state, warm connections, same-region cell):
   ≤100ms P99. São Paulo (lower edge density): Scenario B ≤80ms p99.
2. **DDoS absorbed at edge.** Volumetric attacks (1+ Tbps) absorbed
   by Cloudflare in Year 1-2; absorbed by oyatie-owned Pingora POPs
   + transit-provider RTBH coordination in Year 3+.
3. **TLS 1.3 + PQ hybrid + HTTP/3 universal.** Modern crypto +
   modern transport at every client-facing endpoint; defence
   against harvest-now-decrypt-later attacks.
4. **Cell isolation preserved.** Per ADR-0248, cell failures don't
   cascade; cross-cell traffic is async-first slow path; intra-cell
   fast path stays fast.
5. **Tenant residency preserved.** GeoDNS routes never violate
   ADR-0049 + ADR-0240; EU tenants never traverse US cells without
   explicit consent.
6. **SPIFFE workload identity universal.** Per ADR-0243 Cedar gate
   has a unique workload principal claim on every request; no
   bypass.
7. **In-house migration path documented.** Pingora + PowerDNS
   self-hosted by Year 3-5; ASN + BGP self-managed by Year 4-5;
   matches ADR-0211 in-house preference at scale.
8. **WebSocket + SSE + WebTransport realtime support.** Collaborative
   editing + LLM streaming + voice/video signalling at planetary
   scale.
9. **Hyperscaler shape achieved.** Matches Cloudflare + Google +
   AWS + Microsoft + Apple + Stripe at the network layer; closes
   the `feedback_quality_performance_scalability_bar` requirement.

### Negative

1. **Multi-vendor + self-hosted dual-track for 3+ years.** Operational
   load during Year 1-3 includes Cloudflare zone config + Route 53
   + per-cell Cilium + per-cell SPIRE + per-cell Envoy waypoint
   + future Pingora staging. Mitigation: each component managed via
   Flux Helm releases + Ansible bare-metal playbooks; ops-sre rotation
   covers all.
2. **Edge Cedar evaluation latency.** Per-edge Cedar gate adds ~5ms p99
   (warm V8 isolate, cached bundle) to ~8ms p99 (cold isolate, first
   request) [P5..P95: 2ms–10ms] to first-byte time (evidence:
   docs/performance-budgets/edge-first-byte-50ms-p99.md §2 Scenario
   decomposition). This consumes 10–16% of the 50ms first-byte budget
   in Scenario B; the remaining per-stage budget (TLS+WAF+rate-limit:
   6ms, POP→cell roundtrip: 20ms, cell handler: 19ms) fits within the
   residual. Scenario A (edge cache hit) adds only ~2ms p99 Cedar
   overhead. Scenario C (cold conn, cross-region) is bounded by the
   200ms budget where the Cedar contribution is ≤8ms. Mitigation:
   per-edge Cedar fragment cache with 30-second TTL; PDP returns
   policy version in response so cache invalidation is correct.
3. **Post-quantum interop bumps.** Some clients (IoT, older browsers)
   don't yet support PQ hybrid; offering both classical + hybrid
   suites doubles the handshake surface during 2027 transition window.
4. **BGP / ASN operational specialty.** Year 4+ requires
   network-engineering specialty (BGP + RPKI + transit-provider
   relationships); historically a smaller talent pool than K8s.
   Mitigation: hire 2-3 senior network engineers Year 3-Q4.

### Operational

1. **New µservices:**
   - `microservices/edge-gateway/` — manages Cloudflare zone config
     + Pingora config (Year 3+); cosign-attested config per
     ADR-0223.
   - `microservices/dns/` — manages DNS records (Cloudflare + Route
     53 in Year 1-2; PowerDNS in Year 3+); records signed per
     ADR-0223.
   - `microservices/service-mesh/` — manages Cilium + Istio Ambient
     Helm releases + SPIRE federation trust bundles; cell-local
     Cedar fragments.
2. **Per-cell artifacts:**
   - `iac/helm/<cell>/cilium-values.yaml` — Cilium 1.16 LTS config.
   - `iac/helm/<cell>/istio-ambient-values.yaml` — Istio Ambient
     1.24 LTS config.
   - `iac/helm/<cell>/spire-values.yaml` — SPIRE Server +
     trust bundle federation config.
   - `iac/helm/<cell>/envoy-ingress-values.yaml` — Envoy ingress
     gateway config (TLS 1.3 + HTTP/3 + cipher suites + PQ hybrid
     when enabled).
3. **New CI lanes (advisory until substrates land; BLOCKER post):**
   - `oya-check-network-topology-coherence` — every cell declares
     home_cell + dr_cell + ingress IP + Anycast region.
   - `oya-check-edge-pop-presence` — every public tenant-facing
     surface has at least one edge POP within latency budget.
   - `oya-check-tls13-only` — no TLS 1.0/1.1/1.2 listener in any
     ingress Envoy config.
   - `oya-check-spiffe-svid-coverage` — every Pod has a SPIFFE
     SVID; no anonymous workloads.
   - `oya-check-post-quantum-hybrid-readiness` — Envoy + s2n-tls
     + SPIRE config supports `liboqs` provider (advisory until Year
     2; BLOCKER thereafter).
   - `oya-check-http3-availability` — every public ingress
     advertises HTTP/3 via `Alt-Svc`.
   - `oya-check-webhook-signing` — outbound webhook signing config
     present + dual-signed (HMAC + Ed25519).
4. **Per-µservice changes:**
   - Every µservice declares `mesh_layering.ambient_waypoint:
     true|false` in `manifest.json` (per ADR-0148).
   - Every µservice ships `iac/helm/<ms>/templates/
     ciliumnetworkpolicy.yaml` with default-deny egress + per-
     sibling allow lists.
   - Every µservice ships `iac/helm/<ms>/templates/spiffe-
     identity.yaml` declaring its SPIFFE-ID claim path.
5. **Observability:**
   - Hubble flow records (Tier 1 Cilium) + ztunnel telemetry (Tier
     2 ambient) + waypoint Envoy access logs (Tier 3) + ingress
     Envoy access logs + edge Cloudflare logs all route through OTel
     Collector per ADR-0153.
   - Per-cell + per-POP latency dashboards; per-tenant request
     volumetrics.
   - DDoS event dashboards; volumetric attack absorption rate.
   - PQ hybrid handshake percentage dashboards (track adoption).
6. **Runbooks:**
   - `docs/runbooks/cell-failover-procedure.md` — manual cell
     failover when GeoDNS automation fails.
   - `docs/runbooks/edge-pop-onboarding.md` — POP onboarding
     procedure (Year 3+).
   - `docs/runbooks/post-quantum-rollout.md` — phased PQ hybrid
     enablement.
   - `docs/runbooks/webhook-dead-letter-recovery.md` — handling
     dead webhooks after 24h.

### Sustainability

- **Edge POP power draw.** Each Pingora POP at full utilisation
  draws ~5-15 kW (depending on server count); 30-100 POPs = 150 kW
  to 1.5 MW. Mitigation:
  - **POP colocation site selection prefers low-PUE datacentres**
    (Power Usage Effectiveness < 1.4); Tier-1 IXes (DE-CIX,
    AMS-IX) co-locate in datacentres with PUE 1.2-1.3.
  - **Renewable energy procurement** preferred at each POP;
    Cloudflare + Akamai both 100% renewable per RE100 commitments;
    oyatie matches at Year 4+.
  - **Carbon attribution per-POP via FinOps sustainability tag**
    (ADR-0174); per-tenant carbon attribution rolled up to
    `oyatie.platform-ops` cost centre.
- **Hot-restart and connection reuse** preserve power efficiency
  vs full cold-start architectures.

### Compliance

- **TLS 1.3 + post-quantum for regulators.** EU AI Act + GDPR +
  KR PIPA + CSAP-equivalent + HIPAA + SOX 17a-4 + FedRAMP all
  reference "industry-standard cryptography"; TLS 1.3 + ML-KEM-768
  hybrid is the 2025-2027 industry standard.
- **DNSSEC + DoH/DoT** for client DNS privacy + zone integrity.
- **WAF (Cloudflare + Coraza)** evidence for PCI-DSS 4.0 + OWASP
  CRS compliance.
- **DDoS protection evidence** for ISO 22301 business-continuity
  + SOC 2 Type II CC8.1.
- **SPIFFE/SPIRE evidence** for SOC 2 CC6.1 (logical access) +
  ISO 27001 A.9 (access control).
- **Audit-chain integration:** every cross-cell call emits to
  audit-chain per ADR-0028 + ADR-0145 Invariant 1; cross-provider
  egress emissions tagged sovereign-data-class per ADR-0240.

## Implementation surface

The following artifacts are required for this keystone to be
considered implemented:

| Artifact | Status |
|---|---|
| `/specs/network-topology.json` | NEW — derived from §D |
| `/specs/microservices/edge-gateway.json` | NEW |
| `/specs/microservices/dns.json` | NEW |
| `/specs/microservices/service-mesh.json` | NEW |
| `microservices/edge-gateway/src/cloudflare_zone_manager.rs` | NEW |
| `microservices/edge-gateway/src/pingora_config_manager.rs` | NEW (Year 3+) |
| `microservices/dns/src/cloudflare_dns_sync.rs` | NEW |
| `microservices/dns/src/route53_sync.rs` | NEW |
| `microservices/dns/src/powerdns_authoritative.rs` | NEW (Year 3+) |
| `microservices/service-mesh/src/cilium_config.rs` | NEW |
| `microservices/service-mesh/src/istio_ambient_config.rs` | NEW |
| `microservices/service-mesh/src/spire_federation.rs` | NEW |
| `microservices/policy-engine/fragments/spiffe-id-principal.cedar` | NEW |
| `microservices/policy-engine/fragments/cross-cell-permits.cedar` | NEW |
| `microservices/policy-engine/fragments/cross-provider-permits.cedar` | NEW |
| `microservices/policy-engine/fragments/sovereign-egress-deny.cedar` | NEW |
| Per-cell `iac/helm/<cell>/cilium-values.yaml` | NEW per cell |
| Per-cell `iac/helm/<cell>/istio-ambient-values.yaml` | NEW per cell |
| Per-cell `iac/helm/<cell>/spire-values.yaml` | NEW per cell |
| Per-cell `iac/helm/<cell>/envoy-ingress-values.yaml` | NEW per cell |
| Per-µservice `iac/helm/<ms>/templates/ciliumnetworkpolicy.yaml` | SWEEP — every µservice |
| Per-µservice `iac/helm/<ms>/templates/spiffe-identity.yaml` | SWEEP — every µservice |
| Per-µservice `manifest.json` `mesh_layering.ambient_waypoint` field | SWEEP — every µservice |
| `docs/standards/network-topology.md` | NEW |
| `docs/standards/post-quantum-rollout.md` | NEW |
| `docs/standards/edge-pop-onboarding.md` | NEW |
| `docs/standards/webhook-signing.md` | NEW |
| `docs/runbooks/cell-failover-procedure.md` | NEW |
| `docs/runbooks/edge-pop-onboarding.md` | NEW |
| `docs/runbooks/post-quantum-rollout.md` | NEW |
| `docs/runbooks/webhook-dead-letter-recovery.md` | NEW |
| CI lane `oya-check-network-topology-coherence` | NEW |
| CI lane `oya-check-edge-pop-presence` | NEW |
| CI lane `oya-check-tls13-only` | NEW |
| CI lane `oya-check-spiffe-svid-coverage` | NEW |
| CI lane `oya-check-post-quantum-hybrid-readiness` | NEW |
| CI lane `oya-check-http3-availability` | NEW |
| CI lane `oya-check-webhook-signing` | NEW |

## Verification

- [ ] `dig +short app.oyatie.com @1.1.1.1` from EU IP resolves to
      Frankfurt cell edge ingress IP; from US IP resolves to N.Va
      cell edge ingress IP; from KR IP resolves to Seoul cell edge
      ingress IP.
- [ ] `dig +short app.oyatie.com @8.8.8.8` with Frankfurt cell
      health-checked-down resolves to Dublin cell edge ingress IP
      (EU DR cell, NOT US cell).
- [ ] `nmap --script ssl-enum-ciphers -p 443 app.oyatie.com` shows
      ONLY TLS 1.3 cipher suites (AES_256_GCM_SHA384,
      CHACHA20_POLY1305_SHA256, AES_128_GCM_SHA256) and no TLS 1.2
      or lower.
- [ ] `curl --http3 -v https://app.oyatie.com/health` returns 200
      with `Alt-Svc: h3=":443"` header.
- [ ] TLS handshake to `app.oyatie.com` from a PQ-hybrid-capable
      client (e.g., Cloudflare's PQ test endpoint + boringssl 2024+)
      negotiates X25519MLKEM768 hybrid KEM (Year 2 verification).
- [ ] `oya gate validate network-topology-coherence` exits 0.
- [ ] `oya gate validate tls13-only` exits 0 across all ingress
      Envoy configs.
- [ ] `oya gate validate spiffe-svid-coverage` exits 0 — every Pod
      across every cell has a SPIFFE SVID.
- [ ] `kubectl get svid -A | wc -l` matches `kubectl get pods -A
      --no-headers | wc -l` modulo system pods.
- [ ] Cedar PDP receives SPIFFE-ID claim on every cross-cell call;
      audit-chain emits cross-cell call evidence with SPIFFE-ID +
      tenant_id + Cedar verdict.
- [ ] WireGuard tunnel between AWS-eu-pack and NHN-kr-pack
      established; cross-provider Cedar permit fragment loaded.
- [ ] Outbound webhook delivery from
      `microservices/notifications/` to a test tenant URL carries
      both `X-Oyatie-Signature-HMAC` and `X-Oyatie-Signature-Ed25519`
      headers; signature verification passes with both keys.
- [ ] Idempotency-key header set on every webhook delivery (per
      ADR-0252); retry of same event delivers same idempotency-key.
- [ ] DDoS simulation (synthetic 100 Gbps L4 + L7 attack against
      `app.oyatie.com`) absorbed at edge with zero per-cell ingress
      saturation; mitigation engages within 30 seconds.
- [ ] DNS query under DNSSEC validation: `dig +dnssec +cd
      app.oyatie.com` returns valid RRSIG records.
- [ ] DoH endpoint reachable: `curl --doh-url
      https://cloudflare-dns.com/dns-query
      https://app.oyatie.com/health` returns 200.
- [ ] First-byte latency (TTFB) from Berlin client to
      `app.oyatie.com` ≤ 50ms (P99 over 1000 samples).
- [ ] First-byte latency from Seoul client to `app.oyatie.com` ≤
      50ms.
- [ ] First-byte latency from São Paulo client to `app.oyatie.com`
      ≤ 80ms (lower edge density in South America).
- [ ] HTTP/3 connection migration test: open HTTP/3 session on WiFi;
      switch to LTE; in-flight requests complete without reconnect.
- [ ] WebSocket session pinned to home_cell push gateway; on
      home_cell failure, client SDK reconnects to dr_cell push
      gateway with audit-chain emission of session migration.

## References

### Cilium + service mesh

- **Cilium project — https://cilium.io ; CNCF Graduated 2023.**
  Cilium 1.16 LTS docs 2024 — sidecarless eBPF service mesh + L4
  identity-based policy + Hubble flow observability + ClusterMesh
  for multi-cluster L4 topology + Cilium Gateway API.
- **Cilium ClusterMesh** — https://docs.cilium.io/en/stable/network/clustermesh/.
- **Cilium 1.16 release notes (Aug 2024)** — Gateway API v1.0
  conformance; FQDN-aware egress; eBPF-based BGP control plane.
- **Hubble (Cilium observability)** — https://github.com/cilium/hubble.

### Istio Ambient

- **Istio Ambient docs 2024 — https://istio.io/latest/docs/ambient/.**
  ztunnel (Rust); waypoint (Envoy); AuthorizationPolicy v1.
- **Istio 1.24 LTS release (Nov 2024)** — Ambient mode GA; stable
  for production.
- **Istio + Cilium hybrid deployment** —
  https://istio.io/latest/docs/ambient/install/platform-prerequisites/#cilium.

### Cloudflare edge

- **Cloudflare Pingora open-sourced 2024 —
  https://github.com/cloudflare/pingora** (CC0/MIT). Pingora
  release blog Feb 2024: "Open sourcing Pingora: our Rust framework
  for building programmable network services."
- **Cloudflare Pingora blog (2022)** — "How we built Pingora, the
  proxy that connects Cloudflare to the Internet."
- **Cloudflare Workers — https://workers.cloudflare.com.**
  V8 isolates at ~300 POPs.
- **Cloudflare Post-Quantum to the People (2024)** —
  https://blog.cloudflare.com/post-quantum-to-the-people/. Documents
  X25519MLKEM768 hybrid KEX deployment.
- **Cloudflare HTTP/3 universal rollout (2020+)** —
  https://blog.cloudflare.com/http3-the-past-present-and-future/.
- **Cloudflare DNS Anycast** —
  https://www.cloudflare.com/learning/dns/what-is-anycast-dns/.

### Post-quantum cryptography

- **NIST FIPS 203 — Module-Lattice-Based Key-Encapsulation Mechanism
  Standard (August 2024).** ML-KEM-768 standard publication.
- **NIST FIPS 204 — Module-Lattice-Based Digital Signature Algorithm
  Standard (August 2024).** ML-DSA for signatures.
- **NIST FIPS 205 — Stateless Hash-Based Digital Signature Algorithm
  Standard (August 2024).** SLH-DSA / SPHINCS+.
- **IETF draft-ietf-tls-hybrid-design** — hybrid KEX for TLS 1.3.
- **AWS s2n-tls PQ — https://github.com/aws/s2n-tls.** Supports
  kyber-tls13 hybrid suite since 2024.
- **OpenSSL 3.x + liboqs provider** — https://github.com/open-quantum-safe/openssl.
- **NSA CNSA 2.0 (2022 + 2024 updates)** — Commercial National
  Security Algorithm Suite 2.0 mandates PQ algorithms for NSS
  systems by 2033; transition timeline 2025-2033.
- **Cloudflare PQ deployment blog (2024)** —
  https://blog.cloudflare.com/pq-2024/. Operational deployment of
  X25519MLKEM768 across Cloudflare's edge.

### SPIFFE / SPIRE

- **SPIFFE Specification** — https://github.com/spiffe/spiffe.
  CNCF Graduated 2022.
- **SPIRE 1.10+ docs** — https://spiffe.io/docs/.
- **SPIFFE Federation API spec v0.6** — cross-trust-domain
  bundle federation.

### DNS / Anycast

- **PowerDNS Authoritative + PowerDNS Recursor docs** —
  https://doc.powerdns.com/.
- **AWS Route 53 Anycast** — https://docs.aws.amazon.com/route53/.
- **DNSSEC RFC 4034 + 4035** — DNS Security Extensions.
- **DoH RFC 8484 + DoT RFC 7858** — DNS-over-HTTPS / DNS-over-TLS.
- **RFC 8499 + BCP 219** — DNS terminology + best practices.

### BGP / IP advertisement

- **BGP RFC 4271** — Border Gateway Protocol 4.
- **RPKI RFC 6810 + RFC 8205** — Resource Public Key Infrastructure
  + BGPsec.
- **RTBH RFC 5635 + RFC 7999** — Remotely Triggered Black Hole +
  BLACKHOLE community.
- **RIPE NCC + ARIN + APNIC + KRNIC** — Regional Internet Registries.
- **DE-CIX + AMS-IX + LINX + KINX + JPIX + Equinix Internet Exchange**
  — Tier-1 IXes.

### HTTP/3 + QUIC

- **RFC 9114** — HTTP/3 (June 2022).
- **RFC 9000** — QUIC transport protocol (May 2021).
- **RFC 9001** — Using TLS to secure QUIC.
- **RFC 9002** — QUIC loss detection + congestion control.

### WebSocket + SSE + WebTransport

- **RFC 6455** — WebSocket Protocol.
- **HTML5 SSE** — https://html.spec.whatwg.org/multipage/server-sent-events.html.
- **WebTransport draft** — https://datatracker.ietf.org/doc/draft-ietf-webtrans-overview/.

### WAF

- **OWASP CRS (Core Rule Set)** —
  https://coreruleset.org. OWASP-maintained WAF ruleset.
- **Coraza WAF docs** — https://coraza.io. Apache 2.0 license;
  OWASP-CRS compatible.

### Webhook signing

- **Stripe webhook signing** — https://stripe.com/docs/webhooks/signatures.
- **Slack request signing** — https://api.slack.com/authentication/verifying-requests-from-slack.
- **GitHub webhook signing** — Ed25519 signatures for GitHub Apps
  (https://docs.github.com/en/webhooks).

### Internal portfolio ADRs

- **ADR-0009** — Cell architecture per-tenant per-region.
- **ADR-0044** — Service mesh + mTLS (inherited).
- **ADR-0049** — Cross-region replication + residency.
- **ADR-0117** — OKE parity.
- **ADR-0121** — On-prem K8s stack — kubeadm + containerd + Istio
  + Envoy.
- **ADR-0145** — Inter-microservice communication reform.
- **ADR-0148** — Service mesh canonical: Cilium L3/L4 + Istio
  Ambient L7 (this ADR extends ADR-0148 to cross-cell + cross-
  provider topology).
- **ADR-0149** — API gateway north-south vs service mesh east-west.
- **ADR-0150** — Cursor pagination canonical.
- **ADR-0153** — Observability backplane layering.
- **ADR-0183** — Policy engine separation (Cedar + Kyverno).
- **ADR-0211** — In-house Rust-primary tech stack (Pingora +
  PowerDNS self-hosting target).
- **ADR-0223** — `oya git` drop-in surface with explicit policy
  verbs (certificate management as signed config).
- **ADR-0240** — Sovereign cloud per regional pack (per-pack
  provider matrix; cross-provider mesh).
- **ADR-0241** — DR + business-continuity portfolio policy (per-
  µservice DR tier; failover matrix).
- **ADR-0242** — Oyatie-is-a-tenant doctrine (keystone #1).
- **ADR-0243** — Cedar as universal gate (keystone #2).
- **ADR-0244** — Tenant as universal scoping primitive (keystone
  #3).
- **ADR-0245** — Substrate vs Product layering (keystone #4).
- **ADR-0246** — Policy-engine substrate promotion (keystone #5).
- **ADR-0247** — Self-hosting / self-modification doctrine
  (keystone #6).
- **ADR-0248** — Amazon-shape cellular architecture (keystone #7;
  cross-cell async slow path).
- **ADR-0252** — Idempotency-keys canonical (keystone #11; webhook
  idempotency).

### Auto-memory feedback

- `feedback_quality_performance_scalability_bar` — reinforced;
  hyperscaler-grade.
- `feedback_bominal_inheritance_precedence` — applies; oyatie
  overrides Bominal where they diverge (Cilium-ambient adoption
  diverges from Bominal's Istio-classic inheritance).
- `feedback_autonomous_implementation_artifacts` — reinforced;
  unblocks autonomous edge POP onboarding via runbooks.
- `feedback_no_silent_regression` — reinforced; cross-cell + cross-
  provider failures opaque-to-caller with explicit retry semantics.
- `feedback_oya_git_canonical_2026_05_18` — applies; certificate +
  zone + Helm config managed via oya git signed config.

---

## Appendix A: Hyperscaler-pattern attribution matrix

Every architectural decision in this ADR is attributed to a named
hyperscaler pattern + source + anti-pattern avoided.

| Decision section | Hyperscaler pattern (named) | Source citation | Anti-pattern avoided |
|---|---|---|---|
| D-1 (Anycast + GeoDNS apex) | "Anycast Apex DNS" | Cloudflare DNS Anycast 300+ POPs; AWS Route 53 Anycast; Google Cloud DNS Anycast 100+ POPs | "Single-region DNS" — apex single point of failure |
| D-1 (DNSSEC) | "Zone integrity attestation" | Cloudflare DNSSEC + AWS Route 53 DNSSEC + RFC 4034/4035 | "Unauthenticated DNS" — DNS spoofing risk |
| D-1 (DoH/DoT) | "Client DNS privacy" | Cloudflare 1.1.1.1 DoH + Google 8.8.8.8 DoH + RFC 8484 + RFC 7858 | "Plaintext DNS surveillance" — ISP DNS surveillance |
| D-2 (Cloudflare Workers + WAF + Bot) | "Planetary Edge POP" | Cloudflare Workers 300+ POPs + Akamai 4000+ POPs + AWS CloudFront 350+ POPs | "Cloud-provider-LB-only" — POP density inadequate (Alt-1 rejection rationale) |
| D-2 (Pingora migration Year 3+) | "Rust-based Edge Proxy at Scale" | Cloudflare Pingora open-source 2024 (Rust; powers Cloudflare's own edge) | "Forever vendor edge" — vendor lock-in at hyperscaler scale |
| D-3 (TLS 1.3 only) | "Modern Crypto at Edge" | Mozilla SSL Configuration Generator "modern"; Stripe API TLS 1.3 only; Cloudflare zones TLS 1.3 by default | "TLS 1.2 downgrade attack surface" — POODLE, BEAST, CRIME, HEARTBLEED legacy |
| D-3 (Per-cell ingress Envoy) | "L7 Ingress Termination" | Istio + Envoy at GKE + EKS + AKS | "Cloud-LB-as-L7" — limited mutability surface |
| D-3 (cert mgmt via signed config) | "Certificate-as-Code" | ADR-0223 oya git signed config + cosign attestations + Let's Encrypt ACME automation | "Ad-hoc cert deployment" — outage from cert rotation drift |
| D-4 (PQ hybrid ML-KEM-768) | "Post-Quantum Hybrid KEX" | Cloudflare X25519MLKEM768 2024 + AWS s2n-tls kyber-tls13 + NIST FIPS 203 | "Harvest-now-decrypt-later" — recorded traffic decrypted at CRQC arrival |
| D-5 (HTTP/3 universal) | "Modern Transport at Edge" | Cloudflare HTTP/3 since 2020 + Google HTTP/3 universal + AWS CloudFront HTTP/3 since 2022 | "TCP-only" — head-of-line blocking + slow session resumption |
| D-6 (Cilium ambient + Istio Ambient) | "Layered L3/L4 + L7 mesh" | ADR-0148 + Google GKE Dataplane V2 + Solo.io reference architecture | "Sidecar tax" — Istio-classic 2× CPU + 30% memory overhead |
| D-7 (SPIFFE/SPIRE workload identity) | "Workload identity primitive" | Google ALTS + SPIFFE/SPIRE CNCF Graduated 2022 + IRSA at AWS EKS | "Static service accounts" — credentials stolen and used indefinitely |
| D-8 (cross-cell async slow path) | "Cellular Architecture Async" | Amazon cells (per Werner Vogels 2019 re:Invent + Pat Helland) | "Cross-cell sync hot path" — cell failure cascades |
| D-9 (cross-provider WireGuard) | "Per-pair encrypted tunnel" | Google cross-cloud Anthos + IBM Satellite | "Public-internet plaintext cross-provider" — eavesdropping risk |
| D-10 (Cilium default-deny egress) | "Zero-trust egress" | NIST SP 800-207 zero-trust architecture + AWS VPC security groups + Cilium 1.14+ identity policy | "Default-allow egress" — exfiltration risk |
| D-11 (per-cell L4 + L7 LB) | "Layered load balancing" | Envoy + Cilium kube-proxy replacement + GKE service mesh | "Single-tier LB" — limited control |
| D-12 (Year 5+ self-managed BGP) | "Own ASN + RPKI" | Cloudflare ASN 13335 + Google ASN 15169 + RIPE NCC ROA | "Forever-cloud-BGP" — vendor lock-in at planetary scale |
| D-13 (GeoDNS to home_cell with residency fallback) | "Tenant-aware residency routing" | AWS Route 53 GeoDNS + Cloudflare GeoSteering | "Residency-blind failover" — EU data crosses Atlantic on EU cell failure |
| D-14 (OpenAPI 3.2 + gRPC + AsyncAPI; GraphQL BFF rejected by ADR-0565) | "Multi-protocol API surface" | Stripe API (REST 3.x) + Google gRPC + Slack AsyncAPI | "Single-protocol bottleneck" — REST-only forces RPC-over-REST patterns |
| D-15 (SSE primary + WS bidirectional + WebTransport) | "Realtime push tier" | Slack WebSocket + Discord SSE + ChatGPT SSE for token stream + Zoom WebTransport | "Polling" — high request volume + latency |
| D-16 (Dual-signed webhook + saga + idempotency) | "Webhook reliability triplet" | Stripe webhook HMAC + Slack request signing + GitHub Ed25519 + AWS EventBridge retry + Stripe idempotency | "Single-sig + fire-and-forget" — replay attacks + lost events |
| D-17 (Pingora migration plan) | "Phased self-hosting migration" | Stripe + Cloudflare + GitHub all reached self-hosted edge by year 5-7 | "Forever-hosted-edge" — vendor margin compounds at planetary scale |
| D-18 (PowerDNS migration plan) | "Phased self-hosting DNS" | Cloudflare DNS + Akamai + Hurricane Electric all self-host authoritative DNS | "Forever-hosted-DNS" — vendor lock-in for foundational primitive |

---

## Appendix B: Worked example — Berlin user → Cloudflare edge → EU cell → Cedar gate → response

To illustrate the end-to-end network shape concretely, here is a
worked example tracing a request from a Berlin user (a knowledge
worker employed by a tenant `acme-corp` whose home_cell is the
Frankfurt EU cell) opening the Workflow Studio canvas at
`https://acme-corp.oyatie.com/workflow-studio`.

**Step 1: User loads the page.**

The user types `acme-corp.oyatie.com/workflow-studio` in their
browser. The browser performs DNS resolution via the system
resolver (typically the OS's stub resolver pointing to a recursive
DNS provider — corporate DoH at the user's employer, or
Cloudflare 1.1.1.1, or Google 8.8.8.8).

**Step 2: DNS resolution.**

The recursive resolver queries the authoritative DNS for
`acme-corp.oyatie.com`. Cloudflare DNS (Year 1-2) responds with a
GeoDNS-routed CNAME pointing at the closest Cloudflare edge POP for
the user's IP geolocation: `acme-corp.oyatie.com.cdn.cloudflare.net`
→ `<cloudflare-berlin-pop-anycast-ip>` (TTL 60 seconds).

Per D-13, the GeoDNS lookup also considers the tenant's home_cell
preference (encoded in a TXT record at
`_oyatie-cell.acme-corp.oyatie.com`). The Cloudflare edge POP holds
a route table mapping `acme-corp` → Frankfurt EU cell ingress IP
(`cell-frankfurt-1.eu.oyatie.com`).

**Step 3: Edge POP (Cloudflare, Year 1-2).**

The browser opens a TCP connection on port 443 to the Cloudflare
Berlin POP Anycast IP. The browser advertises:
- TLS 1.3 ClientHello (cipher suites: AES_256_GCM_SHA384,
  CHACHA20_POLY1305_SHA256, AES_128_GCM_SHA256).
- TLS 1.3 supported_groups: X25519MLKEM768 (PQ hybrid; Year 2+),
  X25519, secp256r1.
- TLS 1.3 ALPN: `h3`, `h2`, `http/1.1`.
- SNI: `acme-corp.oyatie.com`.

The POP terminates TLS 1.3 in ~3ms with PQ hybrid KEX (Year 2+).
The POP returns the negotiated cipher suite + selected ALPN protocol
(`h3` if the browser supports HTTP/3 via Alt-Svc cache; else `h2`).

The Cloudflare edge:
1. **DDoS check** — request volume from the source IP within the
   60-second rate-limit budget per source IP (default 1000 req/s
   per IP); if exceeded, request rate-limited at edge.
2. **WAF check** — OWASP CRS rules evaluated; SQL-injection +
   XSS + path-traversal patterns blocked.
3. **Bot check** — Cloudflare Bot Management evaluates UA + TLS
   fingerprint + behavioural signals; if scored as bot, hCaptcha
   challenge served.
4. **Edge Cedar fragment** — per-tenant rate limit Cedar fragment
   evaluated; for `acme-corp` tenant, the per-tenant rate limit is
   100 req/s for the Workflow Studio canvas endpoint; if exceeded,
   429 returned at edge.
5. **Origin routing** — based on host (`acme-corp.oyatie.com`) +
   path (`/workflow-studio`), the POP routes upstream to
   `cell-frankfurt-1.eu.oyatie.com` (Frankfurt EU cell ingress).

**Step 4: Cloudflare POP → Frankfurt cell.**

The POP establishes a connection to Frankfurt cell ingress over
Cloudflare's private backbone (Argo Smart Routing) → AWS Direct
→ AWS-Frankfurt cell. Latency Berlin → Frankfurt is
~5-10ms.

The connection is mTLS — Cloudflare's edge POP holds a client cert
issued by oyatie's internal CA; the Frankfurt cell ingress Envoy
validates the cert against the oyatie CA chain.

**Step 5: Frankfurt cell ingress (Envoy).**

Frankfurt cell ingress is an Envoy gateway (per ADR-0148; Istio
Ambient ingress). Envoy:
1. **mTLS validation** — Cloudflare's client cert validated;
   alternative: SPIFFE-ID from a federated trust bundle (if the
   edge POP runs Pingora with SPIRE federation in Year 3+).
2. **HTTP/2 upstream parsing** — request parsed; trace headers
   (`traceparent`) attached for downstream observability per
   ADR-0145 Invariant 2.
3. **Per-tenant route lookup** — `acme-corp.oyatie.com` host +
   `/workflow-studio` path → `workflow-studio-µservice` ClusterIP
   inside the Frankfurt cell.

**Step 6: Cilium L4 policy at workflow-studio Pod.**

Cilium agent on the workflow-studio Pod's node:
1. **L4 identity check** — caller identity (Envoy ingress
   namespace) → callee identity (workflow-studio namespace);
   CiliumNetworkPolicy evaluates; ALLOW.
2. **Hubble flow record emitted** — flow logged to Hubble; OTel
   Collector ships to Tempo + Loki.
3. **Per-Pod SPIRE Agent** issues the workflow-studio Pod's SVID;
   Pod's SPIFFE-ID:
   `spiffe://cell-frankfurt-1.eu-pack.oyatie/ns/workflow-studio/sa/workflow-studio-app`.

**Step 7: ztunnel (Istio Ambient Tier 2).**

ztunnel on the node:
1. **mTLS attach** — outbound from Envoy ingress, inbound to
   workflow-studio Pod, mTLS terminated with workflow-studio's
   SVID.
2. **L4 telemetry** — connection latency + byte count emitted to
   OTel Collector.
3. **Routes to waypoint** — workflow-studio has
   `mesh_layering.ambient_waypoint: true` (the Workflow Studio
   is one of the 5 µservices that handle L7-policed traffic);
   request routed to the Workflow Studio namespace's waypoint.

**Step 8: Waypoint (Istio Ambient Tier 3).**

Waypoint Envoy:
1. **AuthorizationPolicy v1 check** — request method, path,
   headers evaluated against the policy compiled from
   `microservices/workflow-studio/policy/tenant-scope.cedar` by
   the policy-engine µservice.
2. **`ext_authz` to Cedar PDP** — waypoint Envoy calls
   policy-engine Cedar PDP over gRPC. Request:
   - principal: workflow-studio Pod's SVID + acme-corp tenant claim
     (extracted from session cookie).
   - action: `WorkflowStudio::ReadCanvas`.
   - resource: workflow `<workflow-id>`.
   - context: time, source-IP, source-cell.
3. **Cedar verdict** — Cedar evaluates the loaded fragments:
   - Fragment `oyatie.acme-corp.workflow-studio-read` permits
     `acme-corp` principals to read workflows in cell-frankfurt-1.
   - Fragment `oyatie.cross-cell-deny` denies cross-cell read
     attempts to other cells (irrelevant here; same-cell read).
   - Fragment `oyatie.reserved-tenant-namespace` permits
     non-reserved tenants (`acme-corp` is non-reserved).
   - Verdict: **ALLOW**.
4. **Audit-chain seal emitted** — per ADR-0145 Invariant 1,
   workflow-studio µservice emits an audit-chain seal for the
   read action.

**Step 9: workflow-studio responds.**

workflow-studio Pod processes the request:
1. **Ontology projection lookup** — per ADR-0145 Invariant 3,
   workflow-studio queries Ontology for the canvas data (Workflow
   entity + linked Action + Approval entities).
2. **Response composition** — JSON response per OpenAPI 3.2
   contract at `microservices/workflow-studio/contracts/openapi/workflow-canvas.yaml`.
3. **Response sent upstream** — through waypoint → ztunnel →
   ingress Envoy → Cloudflare POP → browser.

**Step 10: Browser renders the canvas.**

Browser receives the JSON response (gzip-compressed; ~50KB after
compression); React/Solid SSR-hydrated canvas renders within ~50ms
of receiving response.

End-to-end latency:
- DNS resolution: 0ms (cached at recursive resolver).
- TLS handshake: 3ms (Cloudflare edge; 1-RTT TLS 1.3; PQ hybrid
  adds ~1ms negligible overhead).
- POP → Frankfurt cell: 7ms.
- Frankfurt cell ingress + ztunnel + waypoint + workflow-studio +
  Ontology read + response: 25ms.
- Frankfurt cell → POP (response): 7ms.
- POP → browser: 3ms.
- **Total: ~45ms TTFB.** P99 budget met.

**Verifying residency.**

Throughout this flow:
- DNS resolved to a Cloudflare EU POP (Berlin).
- Cloudflare → Frankfurt cell over Cloudflare's EU-internal
  backbone + AWS Direct EU-internal.
- All workflow-studio data lives in cell-frankfurt-1.eu-pack
  (AWS Frankfurt; eu-central-1 region).
- Ontology data is projected from workflow-studio's local DB +
  replicated to dr_cell cell-dublin-1.eu-pack per ADR-0049.
- Audit-chain seal stored in the EU pack's audit-chain stream.

**Zero data egress outside EU.** Per ADR-0240, the
`SOVEREIGN_RESTRICTED_EU` data class on this tenant's data is
honoured.

**Verifying SPIFFE-ID claim at every hop.**

- Envoy ingress: client cert from Cloudflare edge (federated trust
  bundle in Year 3+; static CA cert in Year 1-2).
- ztunnel: inbound mTLS with workflow-studio's SVID.
- Waypoint: SPIFFE-ID `spiffe://cell-frankfurt-1.eu-pack.oyatie/
  ns/workflow-studio/sa/workflow-studio-app` validated.
- Cedar PDP: principal claim includes SPIFFE-ID + tenant claim
  (`acme-corp` from session cookie).
- Audit-chain emission: SPIFFE-ID + tenant-ID + Cedar verdict
  emitted as evidence.

**Cross-cell scenario (Step 8 alternative).**

If the user's request had required reading a workflow that lives
in a different cell (e.g., the tenant is multi-cell with sharded
workflows):
- Waypoint would call `cell-dublin-1.eu-pack` ingress over
  cross-cell mTLS with SPIRE-federated trust bundle.
- Cross-cell call would be Cedar-gated at the receiving cell's
  waypoint with `cross-cell-permits.cedar` fragment.
- Idempotency-key per ADR-0252 set on cross-cell write calls.
- Cross-cell call latency adds ~20-30ms (Frankfurt → Dublin); P99
  budget for cross-cell ≤ 200ms still met.

**Cross-provider scenario (further alternative).**

If the user's request had required cross-provider reach (e.g.,
querying a co-tenant's data in NHN Pangyo cell — extreme edge case,
allowed only under explicit tenant cross-provider Cedar permit):
- Waypoint would call NHN-Pangyo ingress over WireGuard tunnel +
  mTLS with cross-provider SPIRE federation.
- Cross-provider call would be Cedar-gated by
  `cross-provider-permits.cedar`; sovereign-data-class denials
  enforced.
- Cross-provider call latency: Frankfurt → Pangyo ≈ 240ms RTT
  (geographic).
- Tenant warned of cross-provider latency premium.

**Observability evidence.**

Throughout this flow, observability is emitted at every layer per
ADR-0153:
- Edge: Cloudflare access log + WAF event log + Bot Management log.
- Ingress: Envoy access log + RED metrics.
- Cilium: Hubble flow record.
- ztunnel: L4 telemetry.
- Waypoint: Envoy access log + Cedar verdict log.
- workflow-studio: application trace (OTel) + audit-chain seal.
- Ontology: query log (per Ontology PRD §Audit + Compliance).

All emissions tagged with `tenant_id=acme-corp`,
`cell=cell-frankfurt-1.eu-pack`, `traceparent=<W3C trace-id>`,
`spiffe-id=<workload SVID>`.

This is the canonical end-to-end shape for every public-facing
request entering the platform.

---

## Naming justification

Every name introduced or ratified by this ADR is validated against BNF v4.1
(`oya-<microservice>[-<bc-tokens>]-<layer>`) and the ADR-0105 13-value canonical
layer enum.

| Name | Layer (ADR-0105) | BNF v4.1 segments | Justification |
|------|-----------------|-------------------|---------------|
| `oya-check-network-topology-coherence` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `network-topology-coherence` | Fitness-check; verifies every cell declares a valid network-topology entry per §D-1; `oya-check-*` flat namespace |
| `oya-check-edge-pop-presence` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `edge-pop-presence` | Fitness-check; verifies every public tenant-facing cell has at least one edge POP entry per §D-2; `oya-check-*` flat namespace |
| `oya-check-tls13-only` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `tls13-only` | Fitness-check; verifies no TLS 1.0/1.1/1.2 listener in any ingress config per §D-3; `oya-check-*` flat namespace |
| `oya-check-spiffe-svid-coverage` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `spiffe-svid-coverage` | Fitness-check; verifies every Pod has SPIFFE SVID workload identity per §D-7; `oya-check-*` flat namespace |
| `oya-check-post-quantum-hybrid-readiness` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `post-quantum-hybrid-readiness` | Fitness-check; verifies Envoy + s2n-tls config for ML-KEM-768 + X25519 hybrid KEX per §D-4; `oya-check-*` flat namespace |
| `oya-check-http3-availability` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `http3-availability` | Fitness-check; verifies every public ingress advertises HTTP/3 per §D-5; `oya-check-*` flat namespace |
| `oya-check-webhook-signing` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `webhook-signing` | Fitness-check; verifies outbound webhook signing config present per §D-16; `oya-check-*` flat namespace |

---

*End of ADR-0253.*
