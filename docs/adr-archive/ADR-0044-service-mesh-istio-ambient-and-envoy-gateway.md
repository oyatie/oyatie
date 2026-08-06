---
id: ADR-0044
status: Superseded
superseded_by: [ADR-700]
doc_status: published
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0044: Service mesh — Istio Ambient mode for east-west, Envoy as edge gateway, mTLS everywhere, per-cell namespace, audited cross-cell traffic

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `cloud`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0003, ADR-0007, ADR-0028, ADR-0042, ADR-0043

---

## Context

Cross-axis traffic is the cohesion thesis at the network layer. If axes call each other over plain HTTP without identity, without encryption, without policy, the cohesion-invariant guarantees from ADR-0001 don't extend to the wire. The pack-of-19 foundation ADRs decided a service mesh is mandatory but did not pin (a) the mesh, (b) the deployment mode, (c) the edge gateway, (d) the per-cell namespace pattern, (e) the cross-cell audit policy.

Istio Ambient mode (released GA 2024) eliminates the per-pod sidecar in favor of per-node `ztunnel` + per-namespace `waypoint` proxies, which sharply reduces resource overhead and operational complexity vs sidecar mode. Envoy at the edge replaces microservice-local gateways (e.g. Caddy-class admission proxies) with the same data plane that backs Istio — uniform configuration, uniform observability.

---

## Decision

We adopt **Istio Ambient mode** as the canonical east-west service mesh; **Envoy** (gateway-class) as the canonical north-south edge gateway; **mTLS everywhere** as the default with per-traffic-type opt-out only via ADR; **per-cell namespace** as the isolation unit; **cross-cell traffic** explicit + Cedar-policied + audit-chained per call.

### Istio Ambient mode

```yaml
# infra/istio/profile.yaml
apiVersion: install.istio.io/v1alpha1
kind: IstioOperator
metadata:
  name: oya-cell-default
spec:
  profile: ambient
  components:
    ztunnel:
      enabled: true        # per-node L4 proxy
    pilot:
      enabled: true
    cni:
      enabled: true
  meshConfig:
    defaultConfig:
      proxyMetadata:
        ISTIO_META_DNS_CAPTURE: "true"
    extensionProviders:
    - name: oya-audit
      envoyExtAuthzGrpc:
        service: audit-emitter.oya-platform.svc.cluster.local
        port: 9000
```

- **ztunnel** (per-node) handles L4 mTLS + identity.
- **waypoint** (per-namespace L7) handles L7 policy (Cedar-policied per ADR-0007), retries, ext-authz to the audit emitter (per ADR-0003).
- No per-pod sidecars — saves CPU + memory + operational surface vs sidecar mode.

### Envoy as edge gateway (supersedes Caddy-class)

```yaml
# infra/envoy-gateway/templates/per-cell-edge.yaml
apiVersion: gateway.networking.k8s.io/v1
kind: Gateway
metadata:
  name: oya-edge-<cell-id>
spec:
  gatewayClassName: envoy
  listeners:
  - name: https
    port: 443
    protocol: HTTPS
    tls:
      mode: Terminate
      certificateRefs:
      - name: oya-edge-cert-<cell-id>
```

- One Envoy gateway per cell (per ADR-0028).
- TLS termination at the edge; mTLS to backends within the cell.
- North-south observability via the same OTel collector as east-west (per ADR-0042).
- Replaces Caddy-class microservice-local edges; uniform data plane reduces fragmentation.

### mTLS everywhere

- Default: **STRICT** PeerAuthentication for every namespace.
- Per-cell CA issued from per-cell HSM partition (per ADR-0043); rotated quarterly per ADR-0043 drill.
- Identity is SPIFFE SVID; per-workload identity tied to KSA (Kubernetes Service Account) which is tied to per-microservice Cedar policy.
- Plain-text traffic permitted only when a documented ADR records a per-traffic-type ADR-tracked extension to the mTLS-everywhere base (e.g., internal observability collector when ext-authz cost is prohibitive). The extension is canonical, not an exception; every plain-text edge is enumerated in the ADR ledger.

### Per-cell namespace as isolation unit

Each cell (per ADR-0028) owns a Kubernetes namespace tree:

```
oya-tenancy-<cell-id>
oya-connect-<cell-id>
oya-<microservice>-<cell-id>
oya-foundry-<cell-id>
oya-cloud-dcops-<cell-id>
oya-search-<cell-id>
oya-ads-<cell-id>
```

Within a cell, intra-namespace and inter-namespace traffic is mTLS-mesh. Outside the cell, traffic is cross-cell (below).

### Cross-cell traffic explicit + audited

A call from one cell to another (e.g. cross-region replication, cross-cell DSR cascade per ADR-0038) is:

1. Routed via the per-cell Envoy egress (not via mesh ambient layer).
2. Authenticated via per-cell SPIFFE SVID exchange.
3. Cedar-policied (per ADR-0007) with a `CrossCellTraffic` action class.
4. Audit-chained per ADR-0003 with both source and destination cell identity + tenant + capability + payload-hash.
5. Subject to per-microservice review for any new cross-cell call type.

### Per-mesh-policy review for new cross-microservice call

Any new cross-microservice network call type (e.g. Workspace Drive → Search RAG endpoint per ADR-0030) requires:

- Mesh policy entry naming source axis, destination axis, payload class, mTLS guarantees.
- Cedar policy gating call.
- Per-call audit-chain emission.
- Cohesion fitness lane confirmation (`oya-governance-cross-microservice-call`) that the call type is registered.

### Per-tenant traffic isolation

Per-tenant traffic is identified via the `oya-tenant-id` header set at the edge gateway (per request authentication via Identity kernel — ADR-0002). Mesh policies enforce per-tenant scoping; a request with tenant T1 cannot reach a backend that serves tenant T2's data, even if the namespaces overlap.

### Per-traffic-type policies

| Traffic class | mTLS | Cedar policy | Audit emit |
|---|---|---|---|
| Within namespace | STRICT | per-microservice policy | sampled (1%) |
| Cross namespace within cell | STRICT | per-microservice policy | sampled (10%) |
| Cross cell within region | STRICT + SPIFFE federation | `CrossCellTraffic` policy | every call |
| Cross region | STRICT + SPIFFE federation + per-region overlay | `CrossRegionTraffic` policy + ADR-0049 residency check | every call |
| North-south (external) | TLS at edge + audit at edge | per-microservice policy + Identity | every call |

### Anti-scope

This ADR does not own the audit chain (per ADR-0003). Does not own the Cedar policy evaluator (per ADR-0007). Does not own per-cell HSM (per ADR-0043). Does not own per-cell observability (per ADR-0042 namespace plumbing).

---

## Consequences

### Positive

- Istio Ambient mode reduces resource overhead vs sidecar by ~40-60% for cells with many small services.
- Envoy at edge + Envoy in mesh = uniform data plane = uniform debugging.
- mTLS everywhere is the only credible posture for cross-microservice calls in a regulated SaaS.
- Per-cell namespace isolation maps cleanly to per-tenant data residency commitments.
- Audited cross-cell traffic gives regulators a single audit chain for cross-region data flow.

### Negative

- Istio is non-trivial; per-cell deployment and upgrade are real ops surface.
- Ambient mode is recent (2024 GA); some patterns are still maturing.
- Envoy edge + Envoy mesh = same expert pool but multiplied configuration burden.
- mTLS-everywhere increases CPU per packet vs plain text; per-microservice benchmarks confirm acceptable but non-zero.

### Operational

- Per-cell mesh control-plane health alarmed.
- Per-cell mTLS cert expiration alarmed (auto-renewed via cert-manager + per-cell CA).
- Per-cell traffic baseline; anomaly detection on cross-cell traffic spikes.
- Per-quarter mesh upgrade drill in non-production cells before production.
- Per-month cross-microservice call inventory review against the registered policy set.

---

## Alternatives considered

### Alternative A — Linkerd

- **Pros:** simpler operational surface; smaller resource footprint.
- **Cons:** less rich L7 policy model; no first-class extension authz (ext-authz) story for our audit emitter requirement.
- **Rejected because:** ext-authz is the mechanism that ties the mesh to the audit chain.

### Alternative B — Istio sidecar mode (not ambient)

- **Pros:** more mature; battle-tested.
- **Cons:** higher resource overhead; per-pod sidecar life-cycle ops surface.
- **Rejected because:** ambient mode is GA and meets our needs at lower overhead.

### Alternative C — Per-axis edge gateways (Caddy / Traefik / NGINX per axis)

- **Pros:** microservice-team independence.
- **Cons:** N edges; per-edge config drift; per-edge observability fragmented.
- **Rejected because:** the cohesion thesis applies to network ingress.

### Alternative D — No mesh; rely on per-service mTLS handshakes

- **Pros:** less infrastructure.
- **Cons:** per-service mTLS handshake is per-service work; identity rotation is per-service; cross-microservice policy is per-service code.
- **Rejected because:** the mesh is the substrate that enforces cross-microservice policy uniformly.

---

## Open questions

1. **Q1.** Per-cell mesh control plane HA — 3 or 5 replicas? Default: 3 for non-regulated cells; 5 for regulated. → ADR-0028.
2. **Q2.** Cross-region SPIFFE federation cadence — per-cell trust bundle refresh hourly or daily? Default: hourly at GA; daily at W+12 if cost matters. → ADR-0043.
3. **Q3.** Edge gateway TLS termination at L4 (TCP passthrough) for some traffic classes (e.g. WebRTC SFU per ADR-0029)? Default: yes; per-traffic-type carve-out documented. → ADR-0029.
4. **Q4.** Per-tenant rate-limiting at edge or at waypoint? Default: edge (cell-wide budgets); waypoint (per-tenant). → ADR-0028.
5. **Q5.** WASM-based custom Envoy filters for per-microservice policy — at GA or W+12? Default: W+12 once Cedar-via-Envoy ext-authz baseline is stable. → ADR-0007.

---

## References

- `docs/PRD.md` §10 (cross-microservice traffic)
- `docs/DESIGN.md` §11 (service mesh), §10 (cross-microservice contracts)
- Istio Ambient Mode docs (CNCF Graduated 2024); SPIFFE / SPIRE specs; Envoy Gateway docs
- ADR-0001 (cohesion), ADR-0003 (audit), ADR-0007 (Cedar + persona tier), ADR-0028 (cloud cells), ADR-0042 (observability), ADR-0043 (HSM + KMS)
