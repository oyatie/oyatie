---
doc_class: Spec
shape: anchor
length_cap: 200
authority_tier: 1
status: Accepted
date: 2026-05-12
adrs_cited: [ADR-0053, ADR-0052, ADR-0054]
purpose: |
  Istio/Envoy mirror primitive, provider-agnostic via service-mesh adapter pattern.
  The mesh-level mechanism dark-launch + per-cell rollback ride on.
planned_enforcement_ref:
  - oya-governance-shadow-diff
related_adrs: [ADR-0044, ADR-0040]
doc_status: published
---

# Traffic-Mirror Specification

> **Status:** pending approval. **Owner:** `axis-foundry`. **Date:** 2026-05-12. **Source:** [ADR-0044 service-mesh strategy](../../../docs/decisions/ADR-0700-ci-admission-live-apex.md).

## 1. Primitive

Traffic-mirror duplicates a request to a secondary target ("shadow upstream") **without affecting the primary response**. The primary upstream's response is returned to the client; the secondary upstream's response is discarded (or captured for diff).

Used as the transport for [`dark-launch-spec.md`](dark-launch-spec.md). Also used during blue/green soak windows to mirror new-path traffic into staging cells.

## 2. Provider-agnostic via adapter

Per [Directive 4](../../plans/MASTERPLAN.md), mirror is exposed by `oya-platform-traffic-mirror-kernel` (NEW) with provider adapters:

- `oya-platform-traffic-mirror-adapter-istio` (NEW) — Istio `VirtualService.mirror` + `mirrorPercentage`. Default per [ADR-0044](../../../docs/decisions/ADR-0700-ci-admission-live-apex.md).
- `oya-platform-traffic-mirror-adapter-envoy-gateway` (NEW) — Envoy Gateway native `RequestMirror` filter.
- `oya-platform-traffic-mirror-adapter-aws-app-mesh` (NEW) — AWS App Mesh shadow (where AWS-native customers run).
- `oya-platform-traffic-mirror-adapter-linkerd` (NEW; future) — Linkerd traffic-split + tap.

Application config never references a specific mesh; mesh-binding is per-cell deployment manifest.

## 3. Canonical Istio mirror (rendered by adapter)

```yaml
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: oya-<axis>-<svc>
spec:
  hosts: [oya-<axis>-<svc>]
  http:
    - route:
        - destination: { host: oya-<axis>-<svc>, subset: blue }
          weight: 100
      mirror: { host: oya-<axis>-<svc>, subset: green }
      mirrorPercentage: { value: 10.0 }
```

Mirror percentage steps with the dark-launch / blue-green stage. Sampling rates per surface defined in [`dark-launch-spec.md`](dark-launch-spec.md) §3.

## 4. Header propagation

Mirrored requests are tagged with `x-oya-shadow: true` (header) and `x-oya-shadow-correlation: <uuid>` (header). Downstream services MUST detect the header and:

1. Skip external side-effects (emails, payment, webhooks).
2. Use a sandbox transaction or shadow store (per [`dark-launch-spec.md`](dark-launch-spec.md) §6).
3. Tag downstream emitted events as shadow (don't propagate to real consumers).

Lane `oya-governance-shadow-diff` includes a static check for `x-oya-shadow` recognition in every service that may receive mirrored traffic.

## 5. Cohort interaction

Mirrored traffic is sampled **only** from cohorts that consent to participate. Stable-regulated tenants are excluded from mirror sampling unless their per-vertical pack explicitly permits ([`stable-cohort-spec.md`](stable-cohort-spec.md) §3). Sampling logic lives in `oya-platform-tenant-cohort-kernel`, queried by the mesh via webhook.

## 6. Failure modes

- **Shadow upstream slow.** Mirror is fire-and-forget by mesh contract; primary response unaffected. Slow shadow accumulates queue → mesh sheds at queue-depth threshold. Configured per-cell.
- **Shadow upstream OOM / crash loop.** Mesh detects via outlier-detection and disables mirror automatically. Emits Sev-2 ticket.
- **Header strip by intermediary.** Forbidden. `oya-governance-shadow-diff` includes a CI check that no Envoy filter strips `x-oya-shadow*`.

## 7. Auditability

Every mirror activation emits a D14 audit-chain entry: which surface, which percentage, which cohort sample, start/stop timestamps, correlation IDs. Mirror activation requires named approver for surfaces tagged `regulated` ([ADR-0034](../../../docs/adr-archive/ADR-0034-per-microservice-data-class-overrides.md)).

## 8. Per-cell scope

Mirror is configured per-cell. A global dark-launch is N per-cell mirror configurations orchestrated by `oya-platform-rollout-controller-kernel`. No global mirror primitive — too easy to mis-target.

## 9. Cost

Mirror = 1.0×–1.1× compute on the shadow path (cold-start amortised by long-running shadow pods). Network egress doubled for mirrored portion. Budgeted per release in `oya-intelligence-cost-budget-kernel`.

## 10. Hyperscaler equivalents

- Istio `mirror` (canonical open-source).
- Envoy `request_mirror_policies` (raw filter).
- AWS App Mesh "shadow listener" patterns.
- NGINX `mirror` directive (commercial).
- Linkerd `service-mirror` (multi-cluster only).
- Google Cloud Load Balancing "shadow backend" (limited regions).

## 11. Compliance gates

- `oya-governance-shadow-diff` (NEW; HIGH).
- `oya-governance-cohort-honor` (NEW; HIGH).

## 12. Lift target

`oyatie/docs/release/traffic-mirror-spec.md` on approval.
