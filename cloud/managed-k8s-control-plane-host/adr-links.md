# ADR links — `oya-managed-k8s-control-plane-host`

| ADR | Title | Relationship |
|-----|-------|--------------|
| [ADR-0376](../../docs/decisions/ADR-0701-monorepo-capability-live-apex.md) | Managed-Kubernetes product surface (two-tier hosted+dedicated) | **Primary authority.** Names this microservice, the two tiers (hosted-Kamaji default / dedicated-Talos premium), the dogfood-first tenant-zero scope, and the deferred GA legs. |
| [ADR-0375](../../docs/decisions/ADR-0701-monorepo-capability-live-apex.md) | Talos + Cluster API + Argo CD fleet substrate | Substrate the product builds on; the dedicated tier IS the ADR-0375 Talos spoke promoted to an SKU. |
| [ADR-0148](../../docs/decisions/ADR-0700-ci-admission-live-apex.md) | Cilium L3/L4 + Istio Ambient L7 (zero overlap) | Mesh layering both tiers inherit; declared in `manifest.json#mesh_layering`. |
| [ADR-0092](../../docs/decisions/ADR-0709-general-live-apex.md) | Workspace dependency-seam policy | Governs the kube-rs / k8s-openapi adapter-only seam isolated to `-adapter-capi`. |
| [ADR-0083](../../docs/decisions/ADR-0700-ci-admission-live-apex.md) | Rust error-handling tier decision | Tier-3 panic-free posture; typed `ProvisioningError` on every fallible path; `#![forbid(unsafe_code)]`. |
| [ADR-0105](../../docs/decisions/ADR-0709-general-live-apex.md) | 13-layer enum + check-family patterns | Crate role/layer assignment (kernel/api/adapter/app) + architecture-boundary edges. |
| [ADR-0131](../../docs/decisions/ADR-0701-monorepo-capability-live-apex.md) | Per-microservice flat layout | Single-concern flat layout; `src/` canonical crate root. |
| [ADR-0132](../../docs/adr-archive/ADR-0132-product-platform-and-bundle-dissolution.md) | No platform/bundle microservices | This is a single-concern microservice (control-plane-host), not a suite. |

## Deferred / follow-on

- `registry/placeholder-debt/adr-follow-ups.yaml#kamaji-provider-live-integration`
  — the live Kamaji `TenantControlPlane` / Talos control-plane CRD reconcile (a
  follow-on ADR owns the real CRD wiring + the concrete Kamaji/CAPI-provider
  version pin in `registry/lts-pins.yaml`).
- `oya-managed-k8s-commercial-ga` (named in ADR-0376 §Decision) — billing, public
  SLA, DPIA, external multi-tenant GA.
