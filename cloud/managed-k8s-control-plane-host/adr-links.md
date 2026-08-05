# ADR links — `oya-managed-k8s-control-plane-host`

| ADR | Title | Relationship |
|-----|-------|--------------|
| [ADR-0376](../../docs/decisions/ADR-0376-managed-kubernetes-product-surface.md) | Managed-Kubernetes product surface (two-tier hosted+dedicated) | **Primary authority.** Names this microservice, the two tiers (hosted-Kamaji default / dedicated-Talos premium), the dogfood-first tenant-zero scope, and the deferred GA legs. |
| [ADR-0375](../../docs/decisions/ADR-0375-talos-capi-argocd-fleet-substrate.md) | Talos + Cluster API + Argo CD fleet substrate | Substrate the product builds on; the dedicated tier IS the ADR-0375 Talos spoke promoted to an SKU. |
| [ADR-0148](../../docs/decisions/ADR-0148-service-mesh-cilium-ambient-layered.md) | Cilium L3/L4 + Istio Ambient L7 (zero overlap) | Mesh layering both tiers inherit; declared in `manifest.json#mesh_layering`. |
| [ADR-0092](../../docs/decisions/ADR-0092-workspace-dependency-seam-policy.md) | Workspace dependency-seam policy | Governs the kube-rs / k8s-openapi adapter-only seam isolated to `-adapter-capi`. |
| [ADR-0083](../../docs/decisions/ADR-0083-rust-error-handling-tier-decision.md) | Rust error-handling tier decision | Tier-3 panic-free posture; typed `ProvisioningError` on every fallible path; `#![forbid(unsafe_code)]`. |
| [ADR-0105](../../docs/decisions/ADR-0105-13-layer-enum-and-check-family-patterns.md) | 13-layer enum + check-family patterns | Crate role/layer assignment (kernel/api/adapter/app) + architecture-boundary edges. |
| [ADR-0131](../../docs/decisions/ADR-0131-per-microservice-flat-layout.md) | Per-microservice flat layout | Single-concern flat layout; `src/` canonical crate root. |
| [ADR-0132](../../docs/decisions/ADR-0132-no-grouping-microservices.md) | No platform/bundle microservices | This is a single-concern microservice (control-plane-host), not a suite. |

## Live-integration pins / follow-on

- `registry/lts-pins.yaml#pins.managed-k8s-control-plane-host` — concrete
  Kamaji/CAPI/Talos/kube/Cilium/Istio live-integration pin authority for this
  service.
- Live/sandbox management-cluster proof remains a separate follow-on artifact:
  target, credentials boundary, objects created/deleted, rollback, and approval.
- `oya-managed-k8s-commercial-ga` (named in ADR-0376 §Decision) — billing, public
  SLA, DPIA, external multi-tenant GA.
