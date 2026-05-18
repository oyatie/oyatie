---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-audit-chain-substrate
impl_plan_id: IP-002-self-slo-manifest
status: pending
owner: axis-audit-chain
acceptance_lanes: [openslo-conformance, per-microservice-layout]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-002: Self-SLO manifests for audit-chain

## Intent

Author OpenSLO v1.0 manifests for audit-chain's own SLIs at `microservices/audit-chain/slos/`. Drives the SLO-gated promotion for the audit-chain µservice itself. SLIs: emit_latency, seal_latency, verify_latency, hsm_avail, cross_channel_root_match.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/audit-chain/slos/emit_latency.openslo.yaml` | create |
| `microservices/audit-chain/slos/seal_latency.openslo.yaml` | create |
| `microservices/audit-chain/slos/verify_latency.openslo.yaml` | create |
| `microservices/audit-chain/slos/hsm_avail.openslo.yaml` | create |
| `microservices/audit-chain/slos/cross_channel_root_match.openslo.yaml` | create |

## Acceptance Gates

```bash
cargo run -p oya-observability-slo-engine-rest -- validate microservices/audit-chain/slos/*.openslo.yaml
cargo run -p oya-dev-cli -- gate validate openslo-conformance --microservice audit-chain
```

## References

- `microservices/audit-chain/PRD.md` §"Performance".
- Observability P01 IP-002 (cross-cutting OpenSLO standard).
- `microservices/observability/PRD.md`.
