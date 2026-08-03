# managed-k8s-control-plane-host — Implementation-Ready Acceptance Criteria

ADR-0376 design-spec-maturity surface (authored 2026-05-28 with the managed-k8s 4-split).

## Acceptance criteria
- The deterministic kernel + port + in-memory adapter surface for this microservice is implementation-ready under ADR-0376.
- Live adapter foundation is present for Kamaji/kube-rs and the dedicated Talos reference path; billing, public SLA emission, external GA, and real sandbox/live cluster action remain explicitly gated follow-ons.
- Cross-microservice ports remain narrow + typed; no out-of-band integration paths.
