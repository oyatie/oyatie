# Managed K8s Tenant Quota — Cost / FinOps

## Cost Model (ADR-0340 declaration)

| Dimension | Value |
|-----------|-------|
| Baseline CPU per tenant | 0.01 vCPU |
| Baseline RAM per tenant | 32 MiB |
| Storage per tenant | 0 (in-memory store; production: shared Postgres) |
| Scaling dimension | per_request (O(1) evaluate) |
| Cell placement class | Tier-3 |

The `evaluate()` function is O(1) with no allocations on the allow path.
No persistent storage is wired in this wave; production Postgres adapter is a
follow-on (registry/placeholder-debt/adr-follow-ups.yaml#adr-0376-billing-emission).
