# managed-k8s-control-plane-host — Implementation-Ready Acceptance Criteria

ADR-0376 design-spec-maturity surface (authored 2026-05-28 with the managed-k8s 4-split).

## Acceptance criteria
- The deterministic kernel + port + in-memory adapter surface for this microservice is implementation-ready under ADR-0376.
- Live-integration legs (Kamaji / kube-rs / billing / SLA emission, as applicable) are honest-deferred per ADR-0376-D3 and registered in `registry/placeholder-debt/adr-follow-ups.yaml`.
- Cross-microservice ports remain narrow + typed; no out-of-band integration paths.
