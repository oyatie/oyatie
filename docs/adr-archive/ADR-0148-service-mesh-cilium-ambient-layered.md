---
id: ADR-0148
status: Superseded
deciders: council-architecture, ops-sre-reliability, ops-security, axis-cloud-k8s, axis-network
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: [ADR-0700]
amended_by: [ADR-0341]
related: [ADR-0009, ADR-0121, ADR-0131, ADR-0145, ADR-0146, ADR-0147, ADR-0149, ADR-0150, ADR-0153, ADR-0182, ADR-0183, ADR-0184, ADR-0185, ADR-0186]
related_specs:
  - /specs/hyperscaler-architecture-invariants.json
  - /specs/per-microservice-flat-layout.json
  - /specs/microservices/manifest-schema.json
amends_note: "Rewrites the prior 'Cilium primary + Istio Ambient Tier-2 opt-in' framing into the canonical hyperscaler shape: a layered separation of concerns with zero feature overlap. Cilium owns CNI / L3 / L4 / kernel-level observability; Istio Ambient owns SPIFFE mTLS / L7 policy via waypoint. Each layer owns one concern. ADR-0174 (regulatory-pack waypoint expansion) is concurrently retired and its substantive content folded into docs/standards/regulatory-pack-authzpolicy-overlays.md."
amendment_2026_05_26: "Version currency fix. The original 'Cilium 1.16 LTS' pin is EOL — upstream Cilium supports only the 3 newest minors (1.17/1.18/1.19 as of 2026-05; 1.16's last patch was 2026-01-13). Canonical pin moved to Cilium 1.19.x (1.19.4). Istio Ambient version generalized to 'track current stable' (GA since 1.24; 1.30.x current). Two layering requirements made explicit in the deployment values (infra/talos/cilium-values.yaml): Hubble relay+metrics enabled (Cilium owns L4 flow observability), and socketLB.hostNamespaceOnly=true so Cilium's socket-LB does not bypass the Ambient ztunnel. Source: cilium/cilium releases + endoflife.date/cilium + istio.io ambient docs, verified 2026-05-26."
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0148 — Service-mesh canonical: Cilium L3/L4 + Istio Ambient L7 (layered globally; zero overlap)

## Status

Accepted (2026-05-18). Adopts a **two-layer service-mesh substrate** in which Cilium and Istio Ambient operate as complementary layers with **zero feature overlap**. Each layer owns exactly one concern.

This ADR REWRITES the prior framing (Cilium Service Mesh primary + Istio Ambient as Tier-2 opt-in for advanced L7) into the canonical hyperscaler shape: layered separation of concerns globally. Every µservice receives both layers; only the waypoint (L7 policy enforcement) is opt-in per-µservice based on whether the µservice actually carries L7-policed traffic. ADR-0174 (regulatory-pack waypoint expansion) is concurrently retired and its substantive content folded into the standards layer at `docs/standards/regulatory-pack-authzpolicy-overlays.md`.

## Context

ADR-0145 (inter-microservice communication reform) replaced the universal-mediator rule with three weaker invariants (audit, tracing, ontology projection) and permitted direct sibling-µservice gRPC under mTLS with Cedar authorization on every call. ADR-0145 names a concrete service-mesh substrate as the operational vehicle for those invariants.

The 5-invariant hyperscaler bar that governs this ADR:

- **Consistency** — same mesh shape across all 32 µservices and all packs (eu, kr, us, us-healthcare, ksa, uae, etc.).
- **Quality** — production-grade L4 + L7 with battle-tested implementations.
- **Scalability** — sidecarless dataplane; per-pod overhead is the hard ceiling.
- **Maintainability** — minimum number of moving parts; each layer rollback-able independently.
- **Integration** — Cedar policy decisions wired into the dataplane via `ext_authz`; SPIFFE workload identity native; W3C trace context propagated.

The hyperscaler reference for layered L3/L4 + L7 mesh:

- **Google Cloud GKE Dataplane V2** uses Cilium for L3/L4 + eBPF observability; **Istio Ambient on GKE** layers SPIFFE mTLS via ztunnel + AuthorizationPolicy via waypoint on top. The two co-exist by design.
- **Cloudflare** runs eBPF (Cilium-style) for L3/L4 + DDoS XDP, then layers Envoy for L7 policy + WAF.
- **AWS EKS + Anthos Service Mesh** — the canonical deployment for regulated workloads layers Cilium CNI + Istio Ambient for the same reason.
- **Solo.io reference architecture** explicitly recommends "Cilium for L3/L4; Istio Ambient for L7" as the production-grade pattern.

The prior framing (Cilium Service Mesh as a full mesh competing with Istio's L7 surface, with Istio Ambient as Tier-2 opt-in only) created **feature overlap**: Cilium Service Mesh implements L7 policy via Envoy-as-host-singleton; Istio Ambient implements L7 policy via waypoint Envoy. Two L7 enforcement paths means two surfaces to author Cedar bindings against, two places to debug an L7 denial, and a mode-switch operator burden. The hyperscaler shape does not tolerate this duality.

## Decision

Oyatie adopts a **layered service-mesh substrate** in which **each layer owns exactly one concern**:

### Layer ownership (canonical; zero overlap)

| Layer | Owner | Responsibilities | Out-of-scope |
|---|---|---|---|
| **Layer 3/4 (kernel-level dataplane)** | **Cilium 1.19.x** (pin 1.19.4) [amended 2026-05-26 — see note] | CNI (pod networking, IPAM); `CiliumNetworkPolicy` (L4 identity-based rules); eBPF flow observability via Hubble; node-to-node WireGuard (or IPsec where WG kernel module unavailable); DDoS protection via eBPF XDP; FQDN-aware egress rules; ClusterMesh for multi-cluster L4 topology | NEVER terminates application TLS; NEVER enforces L7 AuthorizationPolicy; NEVER mutates HTTP response envelopes |
| **Layer 7 (application-aware mesh)** | **Istio Ambient** (GA since 1.24; track current stable, 1.30.x as of 2026-05) | SPIFFE-native workload identity mTLS via **ztunnel** (per-node, Rust); `AuthorizationPolicy` v1 enforcement via **waypoint** (per-namespace Envoy); L7 telemetry (Envoy access logs, stats); traffic shaping (VirtualService canary, mirror, retry, timeout); **`ext_authz` hook** for Cedar PDP per-request authorization; WASM filter extensibility | NEVER handles CNI / pod networking; NEVER replaces Cilium's L4 IDENTITY rules; NEVER duplicates eBPF flow observability (Hubble remains source of truth at L4) |

There is no L7 enforcement path that runs in the Cilium dataplane. Cilium Service Mesh's L7 features (HTTP-aware policy via Envoy-as-host-singleton) are **disabled** in oyatie's deployment; we run Cilium in **CNI-and-L4-only mode**. Istio Ambient owns L7 wholly.

### 3-tier data path

```
+---------------------------------------------------------------+
| Pod A (caller)                                                |
|   gRPC client -> outbound socket                              |
+---------------+-----------------------------------------------+
                |
                v
+---------------------------------------------------------------+
| TIER 1 -- Cilium agent (kernel, eBPF)                         |
|   - CiliumNetworkPolicy L4 IDENTITY check (allow/deny)        |
|   - Hubble flow record emit                                   |
|   - WireGuard node-to-node encryption                         |
|   - FQDN egress check (if egressing to internet/sibling FQDN) |
|   - If allowed -> forward to next hop                         |
+---------------+-----------------------------------------------+
                |
                v
+---------------------------------------------------------------+
| TIER 2 -- Istio Ambient ztunnel (per-node, Rust)              |
|   - SPIFFE-ID workload identity attach (cell-uservice SPIRE)  |
|   - mTLS terminate inbound / originate outbound (HBONE)       |
|   - L4 telemetry hop (latency / byte count)                   |
|   - If destination namespace has a waypoint -> route via T3   |
|   - Else -> direct delivery to destination pod                |
+---------------+-----------------------------------------------+
                |
        +-------+--------+
        |                |
        v                v
+-----------------+  +---------------------------------------+
| Sidecarless     |  | TIER 3 -- Istio Ambient waypoint      |
| fast-path:      |  | (per-namespace, Envoy)                |
| straight to pod |  |   - AuthorizationPolicy v1 evaluate   |
|                 |  |   - ext_authz -> Cedar PDP            |
| uservices that  |  |   - Response envelope mutation        |
| do not enroll a |  |     (regulatory packs, DSA Art. 17,   |
| waypoint stay   |  |      GDPR Art. 22, HIPAA min-necessary|
| sidecarless at  |  |      -- declarative, not in app code) |
| L7.             |  |   - VirtualService canary / mirror    |
+-----------------+  |   - L7 access log -> observability    |
                     +-----------+---------------------------+
                                 |
                                 v
                       +-------------------+
                       | Pod B (callee)    |
                       +-------------------+
```

Traffic without L7 policy needs **bypasses Tier 3** — it gets mTLS via ztunnel (Tier 2) and lands at the destination pod with no L7 hop cost. Traffic that needs L7 policy enforcement (Cedar authorization on read of sensitive entities; regulatory response shaping; canary routing) routes through the waypoint (Tier 3).

Per-µservice waypoint opt-in is declared in the µservice's `manifest.json` under `mesh_layering.ambient_waypoint: true|false`. The default is `false`; the 5 µservices that handle L7-policed traffic (governance, foundry, audit-chain, application, workflow-studio) declare `true`. Every other µservice still gets mTLS (Tier 2) and Cedar app-layer checks; they simply skip the L7 mesh hop.

### Cedar PDP wiring (closes ADR-0145 Invariant 2 at L7)

The waypoint's Envoy `ext_authz` filter calls the governance-µservice's Cedar PDP over gRPC for every request that enters the waypoint. The Cedar PDP returns ALLOW / DENY / DENY-WITH-RESPONSE-SHAPE (regulatory packs); the waypoint enforces the verdict on the wire.

The Cedar fragment source-of-truth lives at `microservices/<ms>/policy/tenant-scope.cedar`. The governance µservice's policy compiler emits two artifacts from the same Cedar source:

- `CiliumNetworkPolicy` (CNP) for Tier-1 L4 IDENTITY rules.
- `AuthorizationPolicy` v1 for Tier-3 L7 waypoint rules.

This keeps single-source-of-truth for authorization while honoring the layer boundary.

## Alternatives considered

### (a) Cilium Service Mesh primary (Cilium Envoy for L7) — REJECTED

- **Pros:** single project; one operator skill; eBPF kernel-level efficiency at L7.
- **Cons:** Cilium's L7 surface is narrower than Envoy-in-Ambient — no first-class `AuthorizationPolicy` v1 CRD, no waypoint-style per-namespace policy boundary, no WASM filter ecosystem match. Cedar binding requires Cilium-specific shim. Response envelope mutation (regulatory packs) is not supported on the Cilium L7 path. The Cilium project's own roadmap positions Cilium Service Mesh L7 as an alternative to Istio for environments that **don't** need rich L7; oyatie's regulatory packs (EU DSA, GDPR Art. 22, HIPAA min-necessary) require rich L7.
- **Rejected**: insufficient L7 surface for regulatory response shaping; weaker AuthorizationPolicy ecosystem; ecosystem mismatch with the Cedar + waypoint pattern that Anthos and Solo.io reference deployments use at hyperscaler scale.

### (b) Istio classic sidecar — REJECTED

- **Pros:** widest production deployment; mature `AuthorizationPolicy` + `PeerAuthentication`; richest L7 surface.
- **Cons:** per-pod Envoy sidecar imposes ~2x CPU perf overhead on request path (documented in Istio Ambient perf benchmarks vs classic sidecar); ~30% additional memory cost per pod (50-200MB RAM x pods). At 32-µservice x N-pods fleet this is fleet-wide-noticeable. Sidecar lifecycle race conditions on pod startup. Sidecar-per-pod conflicts with Cilium's eBPF L4 dataplane (double-encrypt + double-policy paths).
- **Rejected**: ~2x perf overhead + ~30% per-pod memory cost is incompatible with the hyperscaler scalability invariant.

### (c) Istio Ambient as the only mesh (no Cilium) — REJECTED

- **Pros:** single project (Istio); Ambient's ztunnel handles L4 mTLS natively.
- **Cons:** gives up eBPF kernel-level data path. Istio Ambient runs as user-space Envoy + Rust ztunnel; the kernel-level XDP DDoS protection, eBPF flow observability via Hubble, and ClusterMesh L4 topology are NOT in scope. Pod networking (CNI) becomes a separate concern (`flannel` / `calico` / etc.); the kernel-fast-path that Cilium delivers is lost. Hubble's eBPF-attributed drop-cause is unmatched by any user-space Envoy telemetry.
- **Rejected**: gives up the eBPF kernel-level wins that Cilium uniquely delivers.

### (d) Linkerd — REJECTED

- **Pros:** simplest operational footprint; Rust proxy; strong default tracing.
- **Cons:** smaller AuthorizationPolicy ecosystem than Istio; no first-class `ext_authz` hook for Cedar PDP wiring; smaller community + ecosystem; no waypoint-equivalent per-namespace L7 boundary; weaker Gateway API conformance; smaller pool of public hyperscaler reference deployments at oyatie's scale target.
- **Rejected**: weaker AuthorizationPolicy ecosystem; smaller community; missing the `ext_authz` pattern oyatie needs for Cedar binding.

### (e) AWS App Mesh / GCP Anthos Service Mesh — REJECTED

- **Pros:** managed control plane on respective hyperscaler.
- **Cons:** AWS App Mesh is AWS-specific (violates the multi-cloud + on-prem portability invariant from ADR-0121); Anthos is GCP-specific. Oyatie ships in EU + KR + on-prem cells where neither is the default. Vendor lock-in.
- **Rejected**: vendor lock-in conflicts with the open-standard primitive doctrine.

### (f) No mesh — REJECTED

- **Pros:** zero mesh-tier complexity.
- **Cons:** ADR-0145 Invariants 1 + 2 (audit-chain seal emission, OpenTelemetry trace propagation) require per-call enforcement; mTLS rotation, Cedar enforcement, response shaping all collapse to application-tier responsibility per µservice; 33 µservices x N owners means silent regulatory regression on auditor question "show the wire shape of an Annex-III-refused response"; cannot meet ADR-0145's tracing/audit/ontology-projection invariants at fleet scale.
- **Rejected**: shifts mesh responsibility into every application; cannot meet ADR-0145.

### (g) **CHOSEN: Cilium L3/L4 + Istio Ambient L7 layered (zero overlap)**

- **Pros:**
  - Each layer owns one concern; zero feature overlap; auditable layer boundary.
  - Cilium delivers eBPF kernel-level dataplane efficiency at L3/L4 + Hubble observability.
  - Istio Ambient delivers SPIFFE mTLS via ztunnel + rich L7 via waypoint + Cedar `ext_authz` binding + response envelope mutation for regulatory packs.
  - Per-µservice waypoint opt-in: µservices that don't carry L7-policed traffic stay sidecarless at L7 (Tier 3 bypass) and pay zero L7 hop cost.
  - Both projects are CNCF Graduated (Cilium 2023; Istio 2024). Production references at GKE, EKS, Anthos, Solo.io, Bell Canada, Capital One.
  - ztunnel is Rust (Oya-aligned per ADR-0120).
  - Independent rollback: each layer disables independently without taking down the other.
- **Cons:**
  - Two control planes (Cilium operator + istiod). Mitigation: both managed via Flux + Helm releases; both produce CNCF-conformant CRDs; both ship operator runbooks.
  - L7 hop cost when waypoint is enrolled: ~80-150 microseconds per request (measured in upstream Istio Ambient benchmarks). Mitigation: only enrolled µservices pay this; the fast-path bypasses Tier 3.
  - Regulatory-pack response shaping requires waypoint enrollment of those µservices; documented in the standard at `docs/standards/regulatory-pack-authzpolicy-overlays.md`.
- **Accepted as the canonical mesh substrate.**

## Consequences

### Positive

1. **Layer boundary is auditable.** Reviewer can name what runs where in one sentence: "L3/L4 + CNI + flow obs = Cilium; SPIFFE mTLS + L7 policy + response shaping = Istio Ambient." There is no L7 enforcement path inside Cilium; there is no CNI path inside Istio Ambient. Anyone reviewing a denied flow goes to exactly one layer.
2. **Sidecarless dataplane fleet-wide.** No per-pod Envoy. Per-pod L7 cost is zero unless the µservice enrolls a waypoint. Fleet-scale memory savings vs Istio classic sidecar: 50-200MB RAM x pods.
3. **Cedar PDP wires natively at L7 via `ext_authz`.** Cedar fragment source-of-truth emits both CNP (Tier 1) and AuthorizationPolicy (Tier 3); the governance µservice's policy compiler owns the emit path. Closes ADR-0145 Invariant 2 (per-call authorization) on the dataplane, not in application code.
4. **Regulatory packs land as overlays on the universal mesh, not a separate mesh.** Per `docs/standards/regulatory-pack-authzpolicy-overlays.md`, regulatory packs ship AuthorizationPolicy YAML fragments at `microservices/<ms>/iac/helm/<ms>/templates/regulatory-authpolicy-<pack>.yaml`, gated by pack labels, on the same waypoint the µservice already runs. There is no "regulatory mesh" — there is one mesh, with regulatory deltas.
5. **W3C trace context closes ADR-0145 Invariant 2 without sidecar tax.** Hubble's OTel exporter ships flow records to Tempo; waypoint Envoy's access logs ship L7 detail; per-µservice tracing-client-kernel injects `traceparent` on outbound calls.
6. **CNCF-graduated open-standard primitives.** Cilium graduated 2023; Istio graduated 2024. Both are vendor-neutral. No lock-in.

### Negative

1. **Two control planes to operate.** Cilium operator (CNI + L4 mesh + Hubble) + istiod (Istio Ambient control plane). Mitigation: both managed via Flux + Helm; both produce CRDs that the governance µservice's policy compiler emits to; ops-sre-reliability runs a single mesh runbook spanning both layers; on-call rotation gets both layers' alerts.
2. **L7 hop cost when waypoint is enrolled.** ~80-150 microseconds per request at Tier 3 (measured in upstream Istio Ambient benchmarks). Mitigation: only the 5 µservices that handle L7-policed traffic (governance, foundry, audit-chain, application, workflow-studio) enroll a waypoint by default; per-µservice IP can add or remove the enrollment with one manifest field change. Sidecarless fast-path keeps the other 27 µservices at zero L7 hop cost.
3. **Cedar policy compiler emits two artifacts.** CNP for Tier 1 + AuthorizationPolicy for Tier 3 from the same Cedar source. Mitigation: governance µservice's compiler already does CNP; the AuthorizationPolicy emit shape is a parameterized template.
4. **eBPF kernel-version coupling.** Cilium 1.19.x features require Linux kernel >= 5.10 (oyatie's Talos baseline + Debian 13 trixie + Oracle Linux 9 satisfy this; the floor is recorded in the Cilium upstream release notes for the pinned minor).

### Operational

1. ALL µservices ship `iac/helm/<ms>/templates/ciliumnetworkpolicy.yaml` declaring L4 IDENTITY rules.
2. ALL µservices declare `policy/tenant-scope.cedar`. The governance µservice's policy compiler emits CNP (Tier 1) for every µservice; emits AuthorizationPolicy (Tier 3) for µservices whose manifest declares `mesh_layering.ambient_waypoint: true`.
3. Waypoint-enrolled µservices additionally ship `iac/helm/<ms>/templates/istio-waypoint.yaml` and the regulatory-overlay YAML fragments per `docs/standards/regulatory-pack-authzpolicy-overlays.md`.
4. Per-namespace dataplane mode: all namespaces carry the label `istio.io/dataplane-mode=ambient` (ambient is global), and waypoint enrollment is a separate Gateway resource per namespace.
5. ClusterMesh handles cross-cluster L4 topology (Cilium); Ambient handles cross-cluster L7 mTLS via SPIFFE federation trust bundles (cell-µservice).
6. Observability: Hubble flow records (Tier 1) + ztunnel telemetry (Tier 2) + waypoint Envoy access logs (Tier 3) ship to the observability µservice's collector via OTel per ADR-0153 (observability backplane layering).

## In-house roadmap

Per user directive 2026-05-18 ("Wherever possible, we should support in-house tech stack. Like how AWS, Google, Microsoft, Oracle does."), this ADR's components classify as follows:

| Component | Classification | Rationale | In-house Phase 2 plan |
|---|---|---|---|
| **Cilium 1.19.x** (pin 1.19.4) | KEEP (CNCF Graduated 2023) | eBPF kernel dataplane + Hubble obs + ClusterMesh is THE standard for L3/L4 at hyperscaler scale. Datadog, Adobe, Bell Canada, Capital One run it in production. There is no industry-comparable in-house alternative. | None planned. Adapter at `crates/oya-shared-mesh-l4-kernel` wraps Cilium for theoretical swap; the kernel never grows a competing implementation. |
| **Istio Ambient** (track current stable; 1.30.x as of 2026-05) | KEEP (CNCF Graduated 2024) | SPIFFE-native ztunnel + per-namespace waypoint + AuthorizationPolicy v1 is THE standard for L7 mesh. Anthos, GKE, AWS EKS, Solo.io reference deployments all run it. | None planned. Adapter at `crates/oya-shared-mesh-l7-kernel` wraps Istio Ambient for theoretical swap. |
| **SPIFFE / SPIRE** | KEEP (open standard; CNCF Graduated 2022) | Workload identity is THE standard; native to both Cilium and Istio Ambient. | None planned. |
| **Envoy** (referenced data plane) | KEEP (CNCF Graduated 2018) | THE standard proxy at hyperscaler scale. | None planned. |
| **Hubble** | KEEP (Cilium subproject) | Cilium-coupled; same KEEP rationale. | None planned. |

The IS-the-standard test: a component is KEEP when (1) it is CNCF-graduated or comparable Linux-Foundation-hosted, AND (2) the in-house engineering effort to replace it would not produce a materially better outcome — it would produce a parallel copy of the same standard with smaller adoption and weaker security review.

The 2 control planes named in §Consequences (Cilium operator + istiod) are both managed via Flux Helm releases; the operator-skill cost is the cost of running the standard, not an avoidable in-house dependency. Oyatie's in-house value is the **Cedar policy compiler** (per ADR-0183) emitting CNP + AuthorizationPolicy CRs, the **per-µservice CiliumNetworkPolicy** templates, and the **waypoint-enrollment manifest schema** — all Oya-native code running on KEEP-classified standard engines, matching how AWS, Google, Microsoft, Oracle build on upstream.

## Rollback

Each layer rolls back independently without bringing down the other:

- **Tier 3 (waypoint) rollback per µservice:** flip `mesh_layering.ambient_waypoint: false` in the µservice manifest, regenerate, redeploy. The waypoint is removed; the µservice keeps Tier 1 + Tier 2; L7 policy reverts to app-tier Cedar checks only.
- **Tier 2 (ztunnel / ambient) rollback per namespace:** remove the `istio.io/dataplane-mode=ambient` label from the namespace. ztunnel stops mediating that namespace; mTLS reverts to whatever the application directly negotiates. Cilium L4 IDENTITY rules continue.
- **Tier 1 (Cilium L4 mesh) rollback:** Cilium can drop to **CNI-only mode** by disabling the mesh feature flag in the Cilium Helm release. CNI continues; Hubble continues; L4 IDENTITY rules become advisory. Existing connections drain via Cilium's identity-cache flush window.

The two control planes are managed by Flux Helm releases; rollback is `git revert` of the Helm values change followed by Flux reconciliation. No persisted state is lost.

The full operator runbook lives at `docs/operators/ADR-0148-runtime-impact-changelog.md`.

## References

- Cilium project — https://cilium.io ; CNCF Graduated 2023.
- Cilium Service Mesh — https://docs.cilium.io/en/stable/network/servicemesh/ (Cilium L7 disabled in oyatie deployment per layer boundary).
- Cilium + SPIFFE/SPIRE integration — Cilium 1.14+ workload identity guide (used at Tier 1 for L4 identity; ztunnel owns the workload identity attach at Tier 2).
- Hubble (Cilium observability) — https://github.com/cilium/hubble
- Cilium ClusterMesh — multi-cluster L4 topology.
- Istio Ambient mode — https://istio.io/latest/docs/ambient/ ; ztunnel (Rust) per-node; waypoint (Envoy) per-namespace.
- Istio + Cilium hybrid deployment — https://istio.io/latest/docs/ambient/install/platform-prerequisites/#cilium
- Istio `AuthorizationPolicy` v1 CRD — https://istio.io/latest/docs/reference/config/security/authorization-policy/
- Istio `ext_authz` extension — https://istio.io/latest/docs/tasks/security/authorization/authz-custom/
- Gateway API v1.0 — https://gateway-api.sigs.k8s.io/ (waypoint Gateway resource).
- Solo.io reference architecture — Cilium L3/L4 + Istio Ambient L7 layered.
- Google Cloud GKE Dataplane V2 — Cilium-backed CNI; documented.
- Bell Canada / Capital One / Datadog Cilium production references — public case studies.
- ADR-0009 — cells (per-cell K8s deployment scope).
- ADR-0120 — Rust-first on-prem tooling (ztunnel Rust alignment).
- ADR-0121 — on-prem K8s stack (Cilium CNI baseline; kernel floor).
- ADR-0131 — per-microservice flat layout (manifest declares mesh_layering field per µservice).
- ADR-0145 — inter-microservice communication reform (this ADR operationalizes Invariants 1, 2, and the direct-sibling-egress permission via the layered mesh).
- ADR-0146 — distroless non-root container base image (waypoint Envoy uses ADR-0146 base; Cilium agent runs as DaemonSet at node tier, exempt).
- ADR-0147 — container sandboxing runtime ladder (waypoint runs runc by default; sovereign-tier waypoints upgrade to kata-clh-sev-snp per the ladder).
- ADR-0149 — API gateway (north-south) vs service mesh (east-west) separation.
- ADR-0150 — Kubernetes policy engine separation (Cedar app authz vs Kyverno admission).
- ADR-0153 — Observability backplane layering (Hubble + ztunnel + waypoint signals route through OTel Collector).
- `docs/standards/regulatory-pack-authzpolicy-overlays.md` — regulatory packs as AuthorizationPolicy overlays on the universal mesh.
