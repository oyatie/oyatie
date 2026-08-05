# Managed K8s SLA Observability — Cost / FinOps

## Cost Model (ADR-0376 / ADR-0338 declaration)

| Dimension | Value |
|-----------|-------|
| Baseline CPU per tenant | 0.01 vCPU |
| Baseline RAM per tenant | 32 MiB |
| Storage per tenant | 0 for the current deterministic in-memory observation adapter |
| Scaling dimension | per observation summary/read; latest-snapshot summary math is O(1) |
| Cell placement class | Tier-3 |

This ancillary surface is source authority only for the current managed-K8s SLA
observability foundation: a deterministic kernel, tenant/cluster-scoped summary
port, and in-memory adapter used for local verification. It does not claim live
Prometheus or Kubernetes collection, measured production SLO evidence, public SLA
proof, billing export, or enforcement behavior.

Follow-on live collector and durable evidence lanes must provide their own cost
model, persistence/storage budget, review evidence, and claim ceiling before they
can be cited as production observability authority.
