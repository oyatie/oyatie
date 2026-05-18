---
adr_id: finops-portal-002
authored: 2026-05-18
status: accepted
authority_chain: ADR-0064 + ADR-0199
microservice: finops-portal
---

# ADR finops-portal-002 — Cost-attribution label strategy

## Context

Per ADR-0199 D-2 every workload manifest must carry cost-attribution
labels (`cost-center`, `workload-class`, `regulatory-pack`). This
ADR pins how `finops-portal` exposes those labels through:

1. Pod manifest (helm).
2. Process-emitted Prometheus metrics (constant labels).
3. FOCUS export rows (via `Tags` column mapping).
4. Audit-chain seal envelopes (via envelope metadata).

## Decision

The three labels propagate at all four layers via a single source
of truth: the Helm `costAttribution.*` values. The deployment.yaml
template:

1. Sets pod labels via `oya.tenantCostLabels` helper.
2. Sets `OYA_FINOPS_PORTAL_*` env vars on the container.
3. The app crate (IP-006) reads env vars and:
   - registers Prometheus constant labels.
   - injects into audit-chain envelopes.
   - emits into FOCUS export rows.

ServiceMonitor `relabelings` (per `templates/servicemonitor.yaml`)
preserve the labels on every scraped metric series.

## Rationale

Single source of truth eliminates drift between layers. The env-
var bridge from pod labels → process is the standard pattern per
the canonical-base helpers.

## Consequences

- A label changes via Helm overlay; no code change needed.
- Per-pack overlays (`values-kr.yaml`, etc.) override
  `regulatoryPack` cleanly.
- The OpenCost configmap consumes the same labels, closing the
  attribution loop.

## Alternatives considered

- **Code-defined labels**: rejected because every per-pack
  override would require a rebuild.
- **Annotation-only (no labels)**: rejected because annotations
  do not propagate into Prometheus metric series.

## References

- ADR-0064 canonical-base.
- ADR-0199 cost-attribution canonical.
- IP-003 Helm chart bootstrap.
- IP-006 app observability wiring.
