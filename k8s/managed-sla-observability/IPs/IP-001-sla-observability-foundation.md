# IP-001: SLA observability kernel + port + in-memory adapter

## Acceptance
- Pure kernel computes availability, provisioning latency, and error-budget summaries from normalized observations.
- API crate maps `ControlPlaneStatusReport` from `managed-k8s-control-plane-host-api` into SLA snapshots.
- In-memory adapter stores latest snapshots and returns typed `UnknownCluster` errors for missing clusters.
- App service can ingest direct snapshots or read through `ControlPlaneProvisioning::status`.

## Deferrals
- Live Prometheus/Kubernetes collection is intentionally deferred to a future adapter behind the same API port.
